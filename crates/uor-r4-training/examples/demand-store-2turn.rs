//! Build a TWO-TURN demand-bearing dialogue store from the single-turn demand store.
//!
//! ```text
//! demand-store-2turn <source-dir> <output-dir> [--tokenizer <tokenizer.json>] [--seed N]
//!                    [--corrupt-test] [--samples N]
//! ```
//!
//! The single-turn demand store (`demand-store.rs`) teaches answer-form compliance with documents
//! that never require carrying anything across a turn boundary. This tool re-reads that store,
//! splits every document into its question and answer token spans *through the response mask*, and
//! emits two-turn documents in which the second turn is a real follow-up:
//!
//! ```text
//! BOS  User:  <turn 1 question>  \n  Assistant:  <turn 1 answer>  EOS
//!      \n  User:  <turn 2 follow-up>  \n  Assistant:  <turn 2 answer>  EOS
//! mask 0    mask 0                0     mask 0       1    1      0
//!        0       0               0       0            1    1
//! ```
//!
//! The framing copies the prepared chat corpus's multi-turn convention exactly
//! (`uor-r4.literal-role-dialogue/2`, chat manifest `template_rule`): BOS, then turns joined by a
//! single `\n` separator placed before every turn after the first; a turn is `<marker><content>`
//! with `User: `/`Assistant: ` markers; every assistant turn ends with `<|eos|>`; a
//! document-terminal `<|eos|>` is appended only when the final emitted turn is not assistant. The
//! mask rule is the chat manifest's: 1 on each assistant turn's response content and its
//! terminating `<|eos|>`, 0 on BOS, role markers, turn separators and all user content. That
//! convention was checked against the real chat corpus before writing: the gap between consecutive
//! response runs is `\n`(201) `User:`(55,2728,28) … `\n`(201) `Assistant:`(35,560,652,714,28) in
//! 55,839 of 55,840 cases (the one exception is a degenerate empty user turn), and 73,370 of
//! 73,625 documents end at the final assistant turn's masked EOS with no extra document EOS.
//!
//! Two of the single-turn generator's sixteen clauses are per-document there and cannot hold
//! literally for a document with two assistant turns; this tool checks their multi-turn
//! generalisation and prints the literal single-turn counts as an annex:
//!
//! - `document_no_interior_eos` -> no *unmasked* EOS strictly inside a document (the training
//!   path's own rule in `dialogue_episodes.rs`); a completed masked turn EOS is expected interior.
//! - `response_single_run` -> each assistant marker opens exactly one contiguous masked run that
//!   closes at its own EOS, with an unmasked token (or the document end) after it; a two-turn
//!   document may hold one run per turn.
//!
//! Every other clause is the source tool's, applied per document or per response run. The tool
//! additionally re-checks every assistant turn as a literal single-turn document (the unchanged
//! single-turn clause set) so both readings are reported. Nothing is written unless all clause
//! counters are zero; after writing, the store is re-read through `DialogueSplit::load` and the
//! real `EpisodeIndex`, so acceptance is the training path's own check. `--corrupt-test` corrupts
//! one document in memory (never on disk), shows the offending document index and clause, and
//! re-runs the clean validation to show the store itself is untouched.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use uor_r4_core::native_geometric::mmap_corpus::{
    CorpusWriter, MmapCorpusReader, CORPUS_HEADER_SIZE,
};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::dialogue_episodes::{EpisodeContract, EpisodeIndex, SourceSpan};
use uor_r4_training::stack_dialogue::DialogueSplit;

/// The store's declared vocabulary, as the dialogue corpus binding requires.
const VOCAB_SIZE: u32 = 4096;
/// The bound special IDs: BOS 0, EOS 1, UNK 2 (never emitted).
const BOS_ID: u32 = 0;
const EOS_ID: u32 = 1;
const UNK_ID: u32 = 2;
/// A real in-vocabulary filler for the episode contract's padding slot; never emitted.
const PADDING_ID: u32 = 3;
/// Literal `User:` and `Assistant:` under the version-2 protocol.
const USER_MARKER: [u32; 3] = [55, 2728, 28];
const ASSISTANT_MARKER: [u32; 5] = [35, 560, 652, 714, 28];
/// The canonical turn separator before every turn after the first (`"\n"` -> literal `Ċ`, 201).
const SEPARATOR_ID: u16 = 201;
/// Full-prefix episode context, as the retained dialogue study uses.
const EPISODE_CONTEXT: usize = 256;
/// The source store's manifest label and this store's label.
const SOURCE_LABEL: &str = "demand";
const OUTPUT_LABEL: &str = "demand-2turn";
/// This tool's document order/pairing seed; `--seed` overrides it.
const DEFAULT_SEED: u64 = 0x3274_7572_6e31; // "2turn1"
/// Every third document (position % 3 == 2) pairs two independent demand-bearing turns; the other
/// two thirds carry a follow-up whose answer is only available from turn 1.
const INDEPENDENT_PERIOD: usize = 3;
/// How many offending documents the refusal prints before summarizing.
const MAX_REPORTED_OFFENDERS: usize = 20;

/// Every clause the store must satisfy. The names mirror the single-turn generator's, which mirror
/// the training-time checks in `dialogue_episodes.rs`; the counter for each must end at zero.
const CLAUSES: [&str; 16] = [
    "store_mask_length",
    "document_bos_unmasked",
    "document_bos_follows_eos",
    "document_terminal_masked_eos",
    "document_no_interior_eos",
    "document_no_interior_bos",
    "document_vocabulary",
    "document_mask_binary",
    "user_marker_exact",
    "response_single_run",
    "response_nonempty",
    "response_marker_present",
    "response_marker_unmasked",
    "response_marker_exact",
    "response_terminal_eos",
    "response_no_interior_eos",
];

/// The answer shape of a source document, read from its decoded answer text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Word,
    Phrase,
    YesNo,
    Number,
    List,
}

impl Shape {
    fn label(self) -> &'static str {
        match self {
            Shape::Word => "one_word",
            Shape::Phrase => "short_phrase",
            Shape::YesNo => "yes_no",
            Shape::Number => "number",
            Shape::List => "short_list",
        }
    }
}

/// One parsed source document: `BOS User: <question> \n Assistant: <answer> EOS`, split through
/// the response mask, with the answer text decoded for shape classification.
struct SourceDoc {
    /// `User:` marker + `" <question>"` + separator.
    user_turn: Vec<u16>,
    /// Answer content + terminal EOS (the document's single masked run).
    response: Vec<u16>,
    /// Decoded answer content, e.g. `Green.` or `Red, green, blue.`
    answer_text: String,
    shape: Shape,
}

/// One emitted two-turn document with the provenance this report needs.
struct TwoTurnDoc {
    tokens: Vec<u16>,
    mask: Vec<u8>,
    kind: &'static str,
    dependent: bool,
    answer2: String,
}

/// One built turn's two halves.
struct Turn {
    user_turn: Vec<u16>,
    response: Vec<u16>,
}

const REPEAT_WORD: &[&str] = &[
    "Answer with one word only. What word did you just answer with?",
    "Reply with exactly one word. Repeat the word you just gave.",
    "Give a one-word answer. What was your previous answer?",
    "Answer in a single word. Say again the word you just said.",
    "Use one word only. Which word did you use in your last answer?",
    "Respond with just one word. Repeat the word from your previous reply.",
    "Answer with only one word. What was the word you just replied with?",
    "Keep it to one word. What word did you just say?",
];

const REPEAT_PHRASE: &[&str] = &[
    "Answer with the name only. Repeat the answer you just gave.",
    "Give just the name. What name did you just give?",
    "Answer briefly. What was your previous answer?",
    "Answer with a short phrase only. Repeat the phrase you just used.",
    "Give the shortest correct answer. Say again your last answer.",
    "Reply briefly with the name. What was the name you just gave?",
];

const REPEAT_YES_NO: &[&str] = &[
    "Answer yes or no. Repeat the answer you just gave.",
    "Reply with yes or no. What was your previous answer, yes or no?",
    "Say yes or no. Say again the answer you just gave.",
    "Answer with a single word, yes or no. What did you answer before?",
    "Just say yes or no. Repeat your previous answer.",
    "Answer yes or no only. What was your last answer?",
];

/// `(phrasing, asks whether the previous answer was "No")`: the follow-up answer is the polarity
/// of turn 1's answer under the asked question, so it is derived rather than copied.
const FLIP_YES_NO: &[(&str, bool)] = &[
    ("Answer yes or no. Was your previous answer no?", true),
    ("Reply with yes or no. Was your last answer yes?", false),
    ("Say yes or no. Did you just answer no?", true),
    (
        "Answer with a single word, yes or no. Was your previous answer yes?",
        false,
    ),
    ("Just say yes or no. Did you answer yes before?", false),
    ("Answer yes or no only. Was your last answer no?", true),
];

const REPEAT_NUMBER: &[&str] = &[
    "Answer with a number only. What number did you just give?",
    "Reply with just a number. Repeat the number from your last answer.",
    "Answer with digits only. What was your previous answer as a number?",
    "Give the number alone. Say again the number you just gave.",
    "Answer using a single number. What number did you just say?",
    "Reply with the number only. Repeat your previous number.",
];

const REPEAT_LIST: &[&str] = &[
    "Answer with a short list separated by commas. Repeat the list you just gave.",
    "Give a short comma-separated answer. What was the list you just gave?",
    "Answer with three items, comma-separated. Repeat your previous list.",
    "List exactly three items, separated by commas. Say again the list you just gave.",
    "Reply with a list of three items separated by commas. Repeat your last answer.",
];

/// A numeric derivation over turn 1's number: the follow-up answer is computed, not copied.
#[derive(Clone, Copy)]
enum Op {
    Add(u64),
    Sub(u64),
    Mul(u64),
}

const ARITHMETIC: &[(&str, Op)] = &[
    (
        "Answer with a number only. What is that number plus one?",
        Op::Add(1),
    ),
    (
        "Reply with just a number. What is that number plus ten?",
        Op::Add(10),
    ),
    (
        "Answer with a number only. What is that number times two?",
        Op::Mul(2),
    ),
    (
        "Answer with digits only. What is that number minus one?",
        Op::Sub(1),
    ),
    (
        "Answer using a single number. What is that number plus two?",
        Op::Add(2),
    ),
    (
        "Respond with a number and nothing else. What is that number times three?",
        Op::Mul(3),
    ),
];

const COUNT_LIST: &[&str] = &[
    "Answer with a number only. How many items were in the list you just gave?",
    "Reply with just a number. How many items did that list have?",
    "Answer with digits only. What was the number of items in your last answer?",
    "Give the number alone. How many items were in your previous answer?",
];

/// `(phrasing, item index)`; `usize::MAX` means "the last item".
const ITEM_LIST: &[(&str, usize)] = &[
    (
        "Answer with one word only. What was the first item in that list?",
        0,
    ),
    (
        "Reply with exactly one word. What was the second item in that list?",
        1,
    ),
    (
        "Answer with one word only. What was the third item in that list?",
        2,
    ),
    (
        "Give a one-word answer. Which item came first in the list you just gave?",
        0,
    ),
    (
        "Answer in a single word. What was the last item in that list?",
        usize::MAX,
    ),
];

struct Ledger {
    counts: Vec<(&'static str, usize)>,
    offenders: Vec<(usize, &'static str, String)>,
}

impl Ledger {
    fn new() -> Self {
        Self {
            counts: CLAUSES.iter().map(|clause| (*clause, 0usize)).collect(),
            offenders: Vec::new(),
        }
    }

    fn hit(&mut self, clause: &'static str, document: usize, detail: String) {
        if let Some(entry) = self.counts.iter_mut().find(|entry| entry.0 == clause) {
            entry.1 += 1;
        }
        if self.offenders.len() < MAX_REPORTED_OFFENDERS {
            self.offenders.push((document, clause, detail));
        }
    }

    fn offenders(&self) -> usize {
        self.counts.iter().map(|entry| entry.1).sum()
    }
}

struct Args {
    source: PathBuf,
    output: PathBuf,
    tokenizer: PathBuf,
    seed: u64,
    corrupt_test: bool,
    samples: usize,
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&argv) {
        eprintln!("demand-store-2turn: {error}");
        std::process::exit(1);
    }
}

fn run(argv: &[String]) -> Result<(), String> {
    let args = parse_args(argv)?;
    let bytes = fs::read(&args.tokenizer)
        .map_err(|e| format!("read tokenizer {}: {e}", args.tokenizer.display()))?;
    let tokenizer_sha256 = uor_r4_training::sha256_bytes(&bytes);
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or_else(|| {
        format!(
            "{} is not a byte-level BPE tokenizer",
            args.tokenizer.display()
        )
    })?;
    if tokenizer.vocab_size() != VOCAB_SIZE as usize {
        return Err(format!(
            "tokenizer vocabulary {} is not {VOCAB_SIZE}",
            tokenizer.vocab_size()
        ));
    }
    // The marker and separator IDs are empirical, not assumed.
    let user = tokenizer.encode("User:");
    let assistant = tokenizer.encode("Assistant:");
    let separator = tokenizer.encode("\n");
    if user != USER_MARKER.to_vec() || assistant != ASSISTANT_MARKER.to_vec() {
        return Err(format!(
            "tokenizer role markers differ from the bound contract: \"User:\" -> {user:?} \
             (want {USER_MARKER:?}), \"Assistant:\" -> {assistant:?} (want {ASSISTANT_MARKER:?})"
        ));
    }
    if separator != vec![u32::from(SEPARATOR_ID)] {
        return Err(format!(
            "tokenizer turn separator is {separator:?}, want [{SEPARATOR_ID}]"
        ));
    }

    let sources = load_source(&args.source, &tokenizer)?;
    let mut shape_histogram: BTreeMap<&'static str, usize> = BTreeMap::new();
    for source in &sources {
        *shape_histogram.entry(source.shape.label()).or_default() += 1;
    }
    println!(
        "source: {} documents parsed through the response mask from {}",
        sources.len(),
        args.source.display()
    );
    for (label, count) in &shape_histogram {
        println!("  source answer shape {label}: {count}");
    }

    let documents = build_documents(&sources, &tokenizer, args.seed)?;
    let mut stream: Vec<u16> = Vec::new();
    let mut mask: Vec<u8> = Vec::new();
    for document in &documents {
        stream.extend_from_slice(&document.tokens);
        mask.extend_from_slice(&document.mask);
    }

    let mut kind_histogram: BTreeMap<&'static str, usize> = BTreeMap::new();
    for document in &documents {
        *kind_histogram.entry(document.kind).or_default() += 1;
    }
    let dependent = documents.iter().filter(|d| d.dependent).count();
    let response_tokens: usize = mask.iter().filter(|&&value| value == 1).count();
    println!(
        "emitted: {} two-turn documents, {} tokens, {} response tokens, {} response runs",
        documents.len(),
        stream.len(),
        response_tokens,
        2 * documents.len()
    );
    println!(
        "  dependency-carrying follow-ups: {} ({:.4} of documents); independent pairs: {} ({:.4})",
        dependent,
        dependent as f64 / documents.len() as f64,
        documents.len() - dependent,
        (documents.len() - dependent) as f64 / documents.len() as f64
    );
    for (label, count) in &kind_histogram {
        println!("  follow-up kind {label}: {count}");
    }

    // (a) the sixteen clauses over the emitted bytes, multi-turn reading.
    let ledger = validate_documents(&stream, &mask);
    report_ledger("multi-turn document reading", &ledger);
    // (a') the same sixteen clause names with their literal single-turn reading, applied to every
    // assistant turn as its own single-turn document.
    let unit_ledger = validate_turn_units(&documents);
    report_ledger("literal single-turn reading, one unit per assistant turn", &unit_ledger);
    // Annex: why two clauses are read per turn (these are expected non-zero).
    let (multi_run, interior_eos) = literal_single_turn_counts(&stream, &mask);
    println!(
        "annex (literal single-turn clauses over whole two-turn documents, expected non-zero): \
         documents_with_more_than_one_masked_run {multi_run}, documents_with_an_interior_eos \
         {interior_eos}"
    );
    if ledger.offenders() != 0 || unit_ledger.offenders() != 0 {
        for (document, clause, detail) in ledger
            .offenders
            .iter()
            .chain(unit_ledger.offenders.iter())
            .take(MAX_REPORTED_OFFENDERS)
        {
            eprintln!("  document {document} clause {clause}: {detail}");
        }
        return Err(format!(
            "refusing to write: {} document-level and {} per-turn clause offenders; the training \
             path would reject the store",
            ledger.offenders(),
            unit_ledger.offenders()
        ));
    }

    if args.corrupt_test {
        corruption_test(&stream, &mask);
    }

    write_store(&args.output, &stream, &mask, &args, &sources, &tokenizer_sha256)?;
    let population = verify_store(&args.output, &stream, &mask)?;

    let tokens_path = args.output.join("tokens.u16");
    let mask_path = args.output.join("response_mask.u8");
    println!(
        "wrote {} documents to {} ({} tokens, {} mask bytes)",
        documents.len(),
        args.output.display(),
        stream.len(),
        mask.len()
    );
    println!(
        "  tokenizer sha256 {tokenizer_sha256}; seed 0x{:016x}",
        args.seed
    );
    println!(
        "  tokens.u16 sha256 {} ({} bytes incl. {CORPUS_HEADER_SIZE}-byte UORT header)",
        uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len()
    );
    println!(
        "  response_mask.u8 sha256 {} ({} bytes)",
        uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        fs::metadata(&mask_path).map_err(|e| e.to_string())?.len()
    );
    println!(
        "  episode index (training path, {EPISODE_CONTEXT}-token full prefix): documents {}, \
         response runs {}, response tokens {}, eligible responses (admitted) {}, eligible response \
         tokens {}, excluded over context {}, identical prefixes {}",
        population.documents,
        population.response_runs,
        population.response_tokens,
        population.eligible_responses,
        population.eligible_response_tokens,
        population.excluded_over_context,
        population.identical_prefixes,
    );
    println!(
        "  fractions: response/token {:.6}, tokens/document {:.3}, runs/document {:.3}",
        response_tokens as f64 / stream.len() as f64,
        stream.len() as f64 / documents.len() as f64,
        population.response_runs as f64 / documents.len() as f64,
    );

    for (index, document) in documents.iter().take(2).enumerate() {
        let ids: Vec<u32> = document.tokens.iter().map(|&token| u32::from(token)).collect();
        println!(
            "document {index} ({}, turn-2 answer {:?}): {}",
            document.kind,
            document.answer2,
            tokenizer.decode(&ids)
        );
        println!("  tokens: {ids:?}");
        println!("  mask:   {:?}", document.mask);
    }
    if args.samples > 0 {
        let mut shown: BTreeMap<&'static str, usize> = BTreeMap::new();
        println!("samples ({} per follow-up kind):", args.samples);
        for document in &documents {
            let seen = shown.entry(document.kind).or_default();
            if *seen >= args.samples {
                continue;
            }
            *seen += 1;
            let ids: Vec<u32> = document.tokens.iter().map(|&token| u32::from(token)).collect();
            println!(
                "  [{}] dependent={} turn-2 answer {:?}\n    {}",
                document.kind,
                document.dependent,
                document.answer2,
                tokenizer.decode(&ids).replace('\n', " ⏎ ")
            );
        }
    }
    Ok(())
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut source: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut tokenizer: Option<PathBuf> = None;
    let mut seed = DEFAULT_SEED;
    let mut corrupt_test = false;
    let mut samples = 0usize;
    let mut index = 0;
    while index < argv.len() {
        match argv[index].as_str() {
            "--tokenizer" => {
                tokenizer = Some(PathBuf::from(value(argv, &mut index, "--tokenizer")?));
            }
            "--seed" => {
                let raw = value(argv, &mut index, "--seed")?;
                seed = raw
                    .strip_prefix("0x")
                    .map(|hex| u64::from_str_radix(hex, 16))
                    .unwrap_or_else(|| raw.parse::<u64>())
                    .map_err(|_| format!("--seed expects a u64, got {raw}"))?;
            }
            "--corrupt-test" => {
                corrupt_test = true;
            }
            "--samples" => {
                let raw = value(argv, &mut index, "--samples")?;
                samples = raw
                    .parse::<usize>()
                    .map_err(|_| format!("--samples expects a count, got {raw}"))?;
            }
            "--help" | "-h" => {
                println!(
                    "usage: demand-store-2turn <source-dir> <output-dir> [--tokenizer \
                     <tokenizer.json>] [--seed N] [--corrupt-test] [--samples N]"
                );
                std::process::exit(0);
            }
            other if other.starts_with("--") => return Err(format!("unknown flag {other}")),
            other => {
                if source.is_none() {
                    source = Some(PathBuf::from(other));
                } else if output.is_none() {
                    output = Some(PathBuf::from(other));
                } else {
                    return Err(format!("unexpected extra argument {other}"));
                }
            }
        }
        index += 1;
    }
    Ok(Args {
        source: source.ok_or_else(usage)?,
        output: output.ok_or_else(usage)?,
        tokenizer: tokenizer.unwrap_or_else(default_tokenizer),
        seed,
        corrupt_test,
        samples,
    })
}

fn usage() -> String {
    "usage: demand-store-2turn <source-dir> <output-dir> [--tokenizer <tokenizer.json>] \
     [--seed N] [--corrupt-test] [--samples N]"
        .to_string()
}

fn value(argv: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    argv.get(*index)
        .cloned()
        .ok_or_else(|| format!("{flag} needs a value"))
}

fn default_tokenizer() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join("uor-r4-local/entry-scorer-inputs/tokenizer.json")
}

/// Read the single-turn demand store and split every document into its user turn and its masked
/// response, so the source's question and answer tokens are reused exactly as generated.
fn load_source(directory: &Path, tokenizer: &HfBpeTokenizer) -> Result<Vec<SourceDoc>, String> {
    let tokens_path = directory.join("tokens.u16");
    let mask_path = directory.join("response_mask.u8");
    let manifest_path = directory.join("manifest.json");
    let reader = MmapCorpusReader::open(&tokens_path)
        .map_err(|e| format!("open {}: {e}", tokens_path.display()))?;
    let tokens = reader.as_slice();
    let mask = fs::read(&mask_path).map_err(|e| format!("read {}: {e}", mask_path.display()))?;
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|e| format!("read {}: {e}", manifest_path.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;
    if tokens.len() != mask.len() {
        return Err("source tokens and mask differ in length".into());
    }
    if manifest["drops"]["special_token_occurrences"] != 0
        || manifest["files"][0]["special_token_occurrences"] != 0
    {
        return Err("source manifest reports literal special-token text".into());
    }
    if manifest["files"][0]["label"] != SOURCE_LABEL {
        return Err(format!(
            "source manifest label is {}, want {SOURCE_LABEL}",
            manifest["files"][0]["label"]
        ));
    }
    if reader.vocab_size() != VOCAB_SIZE {
        return Err(format!(
            "source vocabulary {} is not {VOCAB_SIZE}",
            reader.vocab_size()
        ));
    }
    if manifest["files"][0]["tokens"].as_u64() != Some(tokens.len() as u64) {
        return Err("source manifest token count differs from the store".into());
    }
    if tokens.first().copied() != Some(BOS_ID as u16)
        || tokens.last().copied() != Some(EOS_ID as u16)
    {
        return Err("source store does not start at BOS and end at EOS".into());
    }

    let mut starts: Vec<usize> = Vec::new();
    for (position, &token) in tokens.iter().enumerate() {
        if u32::from(token) == BOS_ID {
            if position != 0 && u32::from(tokens[position - 1]) != EOS_ID {
                return Err(format!("source BOS at {position} does not follow an EOS"));
            }
            starts.push(position);
        }
    }
    starts.push(tokens.len());

    let mut documents = Vec::with_capacity(starts.len() - 1);
    for document in 0..starts.len() - 1 {
        let (start, end) = (starts[document], starts[document + 1]);
        let document_tokens = &tokens[start..end];
        let document_mask = &mask[start..end];
        // The single masked run is the response; everything before it is the user turn.
        let mut runs = Vec::new();
        let mut offset = 0;
        while offset < document_mask.len() {
            if document_mask[offset] == 1 {
                let run_start = offset;
                while offset < document_mask.len() && document_mask[offset] == 1 {
                    offset += 1;
                }
                runs.push((run_start, offset));
            } else {
                offset += 1;
            }
        }
        if runs.len() != 1 {
            return Err(format!(
                "source document {document} holds {} masked runs, not one",
                runs.len()
            ));
        }
        let (run_start, run_end) = runs[0];
        if run_end != document_tokens.len() {
            return Err(format!(
                "source document {document} has {} tokens after its response run",
                document_tokens.len() - run_end
            ));
        }
        let marker_start = run_start
            .checked_sub(ASSISTANT_MARKER.len())
            .ok_or_else(|| format!("source document {document} has no room for the marker"))?;
        if run_start <= ASSISTANT_MARKER.len()
            || document_tokens[marker_start..run_start] != ASSISTANT_MARKER.map(|id| id as u16)
            || document_tokens[1..4] != USER_MARKER.map(|id| id as u16)
            || document_tokens[marker_start - 1] != SEPARATOR_ID
        {
            return Err(format!(
                "source document {document} does not carry the version-2 framing"
            ));
        }
        if u32::from(document_tokens[run_end - 1]) != EOS_ID {
            return Err(format!(
                "source document {document} response does not end at EOS"
            ));
        }
        let user_turn = document_tokens[1..marker_start].to_vec();
        let response = document_tokens[run_start..run_end].to_vec();
        let answer_ids: Vec<u32> = response[..response.len() - 1]
            .iter()
            .map(|&token| u32::from(token))
            .collect();
        let answer_text = tokenizer.decode(&answer_ids);
        let shape = classify(&answer_text);
        documents.push(SourceDoc {
            user_turn,
            response,
            answer_text,
            shape,
        });
    }
    if documents.is_empty() {
        return Err("source store holds no documents".into());
    }
    Ok(documents)
}

/// The answer shape, read from the decoded answer text (the source store does not carry its form
/// label through the manifest).
fn classify(answer_text: &str) -> Shape {
    let body = answer_text.trim().trim_end_matches('.').trim();
    if body == "Yes" || body == "No" {
        return Shape::YesNo;
    }
    if !body.is_empty() && body.chars().all(|c| c.is_ascii_digit()) {
        return Shape::Number;
    }
    if body.contains(", ") {
        return Shape::List;
    }
    if body.split_whitespace().count() <= 1 {
        Shape::Word
    } else {
        Shape::Phrase
    }
}

/// Pair documents so that two thirds of the emitted pairs carry a follow-up that cannot be
/// answered from turn 2's own text, and one third pair two independent demand-bearing turns.
fn build_documents(
    sources: &[SourceDoc],
    tokenizer: &HfBpeTokenizer,
    seed: u64,
) -> Result<Vec<TwoTurnDoc>, String> {
    let count = sources.len();
    let order = shuffled_order(count, seed);
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut number_turn = 0usize;
    let mut yes_no_turn = 0usize;
    let mut list_turn = 0usize;
    let mut documents = Vec::with_capacity(count);
    for (position, &first) in order.iter().enumerate() {
        let turn1 = &sources[first];
        let independent = position % INDEPENDENT_PERIOD == INDEPENDENT_PERIOD - 1;
        let (turn2, kind, dependent, answer2) = if independent {
            let partner = order[(position + count / 2) % count];
            if partner == first {
                return Err("independent partner resolved to the same document".into());
            }
            let partner_doc = &sources[partner];
            (
                Turn {
                    user_turn: partner_doc.user_turn.clone(),
                    response: partner_doc.response.clone(),
                },
                "independent_question",
                false,
                partner_doc.answer_text.clone(),
            )
        } else {
            let draw = splitmix64(&mut state);
            match turn1.shape {
                Shape::Word => repeat_turn(
                    tokenizer,
                    turn1,
                    REPEAT_WORD[draw as usize % REPEAT_WORD.len()],
                ),
                Shape::Phrase => repeat_turn(
                    tokenizer,
                    turn1,
                    REPEAT_PHRASE[draw as usize % REPEAT_PHRASE.len()],
                ),
                Shape::YesNo => {
                    yes_no_turn += 1;
                    if yes_no_turn % 2 == 0 {
                        repeat_turn(
                            tokenizer,
                            turn1,
                            REPEAT_YES_NO[draw as usize % REPEAT_YES_NO.len()],
                        )
                    } else {
                        flip_turn(tokenizer, turn1, draw as usize % FLIP_YES_NO.len())?
                    }
                }
                Shape::Number => {
                    number_turn += 1;
                    if number_turn % 2 == 0 {
                        repeat_turn(
                            tokenizer,
                            turn1,
                            REPEAT_NUMBER[draw as usize % REPEAT_NUMBER.len()],
                        )
                    } else {
                        arithmetic_turn(tokenizer, turn1, draw as usize % ARITHMETIC.len())?
                    }
                }
                Shape::List => {
                    list_turn += 1;
                    match list_turn % 3 {
                        0 => repeat_turn(
                            tokenizer,
                            turn1,
                            REPEAT_LIST[draw as usize % REPEAT_LIST.len()],
                        ),
                        1 => count_turn(tokenizer, turn1, draw as usize % COUNT_LIST.len())?,
                        _ => item_turn(tokenizer, turn1, draw as usize % ITEM_LIST.len())?,
                    }
                }
            }
        };
        let mut tokens: Vec<u16> = Vec::new();
        let mut mask: Vec<u8> = Vec::new();
        push(&mut tokens, &mut mask, &[BOS_ID as u16], 0);
        push(&mut tokens, &mut mask, &turn1.user_turn, 0);
        push(
            &mut tokens,
            &mut mask,
            &ASSISTANT_MARKER.map(|id| id as u16),
            0,
        );
        push(&mut tokens, &mut mask, &turn1.response, 1);
        // Every turn after the first is preceded by the single `\n` separator.
        push(&mut tokens, &mut mask, &[SEPARATOR_ID], 0);
        push(&mut tokens, &mut mask, &turn2.user_turn, 0);
        push(
            &mut tokens,
            &mut mask,
            &ASSISTANT_MARKER.map(|id| id as u16),
            0,
        );
        push(&mut tokens, &mut mask, &turn2.response, 1);
        documents.push(TwoTurnDoc {
            tokens,
            mask,
            kind,
            dependent,
            answer2,
        });
    }
    Ok(documents)
}

/// A recall follow-up: turn 2 repeats turn 1's answer, so the answer exists only in turn 1.
fn repeat_turn(
    tokenizer: &HfBpeTokenizer,
    source: &SourceDoc,
    phrasing: &str,
) -> (Turn, &'static str, bool, String) {
    (
        Turn {
            user_turn: user_turn_from_text(tokenizer, phrasing),
            response: source.response.clone(),
        },
        "dependent_recall",
        true,
        source.answer_text.clone(),
    )
}

/// A polarity follow-up: turn 2 asks whether turn 1's yes/no answer was a given polarity, so the
/// answer is derived from turn 1's answer rather than copied from it.
fn flip_turn(
    tokenizer: &HfBpeTokenizer,
    source: &SourceDoc,
    pick: usize,
) -> Result<(Turn, &'static str, bool, String), String> {
    let (phrasing, asks_no) = FLIP_YES_NO[pick];
    let previous = source.answer_text.trim().trim_end_matches('.').trim();
    let was_no = match previous {
        "Yes" => false,
        "No" => true,
        other => return Err(format!("answer {other:?} is not yes or no")),
    };
    let answer = if was_no == asks_no { "Yes." } else { "No." }.to_string();
    Ok((
        Turn {
            user_turn: user_turn_from_text(tokenizer, phrasing),
            response: response_from_text(tokenizer, &answer),
        },
        "dependent_polarity",
        true,
        answer,
    ))
}

/// A numeric follow-up: turn 2 asks for a value derived from turn 1's number.
fn arithmetic_turn(
    tokenizer: &HfBpeTokenizer,
    source: &SourceDoc,
    pick: usize,
) -> Result<(Turn, &'static str, bool, String), String> {
    let body = source.answer_text.trim().trim_end_matches('.').trim();
    let value: u64 = body
        .parse()
        .map_err(|_| format!("answer {:?} is not a number", source.answer_text))?;
    // A zero answer cannot be decremented; rotate to the next derivation that stays natural.
    for offset in 0..ARITHMETIC.len() {
        let (phrasing, op) = ARITHMETIC[(pick + offset) % ARITHMETIC.len()];
        let derived = match op {
            Op::Add(addend) => Some(value + addend),
            Op::Mul(factor) => Some(value * factor),
            Op::Sub(subtrahend) => value.checked_sub(subtrahend),
        };
        if let Some(derived) = derived {
            let answer = format!("{derived}.");
            return Ok((
                Turn {
                    user_turn: user_turn_from_text(tokenizer, phrasing),
                    response: response_from_text(tokenizer, &answer),
                },
                "dependent_arithmetic",
                true,
                answer,
            ));
        }
    }
    Err(format!("no natural-number derivation applies to {value}"))
}

/// A count follow-up: turn 2 asks how many items turn 1's list held.
fn count_turn(
    tokenizer: &HfBpeTokenizer,
    source: &SourceDoc,
    pick: usize,
) -> Result<(Turn, &'static str, bool, String), String> {
    let items = list_items(&source.answer_text)?;
    let answer = format!("{}.", items.len());
    Ok((
        Turn {
            user_turn: user_turn_from_text(tokenizer, COUNT_LIST[pick]),
            response: response_from_text(tokenizer, &answer),
        },
        "dependent_count",
        true,
        answer,
    ))
}

/// An item follow-up: turn 2 asks for one named position of turn 1's list. Every list item in the
/// source store is one word, so the demanded one-word answer holds; a multi-word item would make
/// the demanded form wrong and is refused here.
fn item_turn(
    tokenizer: &HfBpeTokenizer,
    source: &SourceDoc,
    pick: usize,
) -> Result<(Turn, &'static str, bool, String), String> {
    let (phrasing, index) = ITEM_LIST[pick];
    let items = list_items(&source.answer_text)?;
    let resolved = if index == usize::MAX {
        items.len() - 1
    } else {
        index
    };
    let item = items
        .get(resolved)
        .ok_or_else(|| format!("list {:?} has no item {resolved}", source.answer_text))?;
    if item.split_whitespace().count() != 1 {
        return Err(format!("list item {item:?} is not one word"));
    }
    let answer = format!("{item}.");
    Ok((
        Turn {
            user_turn: user_turn_from_text(tokenizer, phrasing),
            response: response_from_text(tokenizer, &answer),
        },
        "dependent_item",
        true,
        answer,
    ))
}

fn list_items(answer_text: &str) -> Result<Vec<String>, String> {
    let body = answer_text.trim().trim_end_matches('.').trim();
    let items: Vec<String> = body
        .split(", ")
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect();
    if items.len() < 2 {
        return Err(format!("answer {answer_text:?} is not a comma-separated list"));
    }
    Ok(items)
}

/// `User: <text> \n`, the user half of one turn.
fn user_turn_from_text(tokenizer: &HfBpeTokenizer, text: &str) -> Vec<u16> {
    let mut tokens: Vec<u16> = USER_MARKER.map(|id| id as u16).to_vec();
    tokens.extend(encode_with(tokenizer, &format!(" {text}")));
    tokens.push(SEPARATOR_ID);
    tokens
}

/// `" <answer>" EOS`, the masked response of one assistant turn.
fn response_from_text(tokenizer: &HfBpeTokenizer, answer: &str) -> Vec<u16> {
    let mut tokens = encode_with(tokenizer, &format!(" {answer}"));
    tokens.push(EOS_ID as u16);
    tokens
}

fn encode_with(tokenizer: &HfBpeTokenizer, text: &str) -> Vec<u16> {
    tokenizer
        .encode(text)
        .into_iter()
        .map(|id| u16::try_from(id).unwrap_or(u16::MAX))
        .collect()
}

fn push(tokens: &mut Vec<u16>, mask: &mut Vec<u8>, ids: &[u16], selected: u8) {
    tokens.extend_from_slice(ids);
    mask.extend(std::iter::repeat(selected).take(ids.len()));
}

/// Deterministic Fisher-Yates over the source order, so the pairing is fixed by `seed`.
fn shuffled_order(count: usize, seed: u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..count).collect();
    let mut state = seed;
    for index in (1..count).rev() {
        let pick = (splitmix64(&mut state) % (index as u64 + 1)) as usize;
        order.swap(index, pick);
    }
    order
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

/// Maximal masked runs of one document, computed from the byte mask alone.
fn mask_runs(mask: &[u8]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut offset = 0;
    while offset < mask.len() {
        if mask[offset] == 1 {
            let start = offset;
            while offset < mask.len() && mask[offset] == 1 {
                offset += 1;
            }
            runs.push((start, offset));
        } else {
            offset += 1;
        }
    }
    runs
}

/// Token positions of every BOS, plus the end of the stream.
fn document_starts(stream: &[u16]) -> Vec<usize> {
    let mut starts: Vec<usize> = Vec::new();
    for (position, &token) in stream.iter().enumerate() {
        if u32::from(token) == BOS_ID {
            starts.push(position);
        }
    }
    starts.push(stream.len());
    starts
}

/// The sixteen clauses over the flat stream, with the two multi-turn generalisations described in
/// the module comment. Used both for the emitted bytes and (on a synthetic single-turn stream) for
/// every assistant turn as a literal single-turn document.
fn validate_documents(stream: &[u16], mask: &[u8]) -> Ledger {
    let mut ledger = Ledger::new();
    if stream.len() != mask.len() {
        ledger.hit(
            "store_mask_length",
            0,
            format!("{} tokens against {} mask bytes", stream.len(), mask.len()),
        );
        return ledger;
    }
    if stream.is_empty() {
        ledger.hit("document_bos_unmasked", 0, "empty stream".to_string());
        return ledger;
    }

    let mut starts: Vec<usize> = Vec::new();
    for (position, &token) in stream.iter().enumerate() {
        if u32::from(token) == BOS_ID {
            if position != 0 && u32::from(stream[position - 1]) != EOS_ID {
                ledger.hit(
                    "document_bos_follows_eos",
                    starts.len(),
                    format!("BOS at {position} follows token {}", stream[position - 1]),
                );
            }
            starts.push(position);
        }
    }
    if u32::from(stream[0]) != BOS_ID {
        ledger.hit(
            "document_bos_unmasked",
            0,
            format!("first token is {} not BOS", stream[0]),
        );
    }
    starts.push(stream.len());

    for document in 0..starts.len() - 1 {
        let (start, end) = (starts[document], starts[document + 1]);
        let tokens = &stream[start..end];
        let mask = &mask[start..end];
        let last = tokens.len() - 1;

        if u32::from(tokens[0]) != BOS_ID || mask[0] != 0 {
            ledger.hit(
                "document_bos_unmasked",
                document,
                format!(
                    "document start {start} is {} with mask {}",
                    tokens[0], mask[0]
                ),
            );
        }
        if u32::from(tokens[last]) != EOS_ID || mask[last] != 1 {
            ledger.hit(
                "document_terminal_masked_eos",
                document,
                format!("final token {} carries mask {}", tokens[last], mask[last]),
            );
        }
        // Multi-turn reading: a completed turn's EOS is masked and expected; only an *unmasked*
        // EOS strictly inside a document violates the training path's rule.
        if tokens[..last]
            .iter()
            .enumerate()
            .any(|(offset, &token)| u32::from(token) == EOS_ID && mask[offset] == 0)
        {
            ledger.hit(
                "document_no_interior_eos",
                document,
                "an unmasked EOS appears before the document end".to_string(),
            );
        }
        if tokens[1..].iter().any(|&token| u32::from(token) == BOS_ID) {
            ledger.hit(
                "document_no_interior_bos",
                document,
                "a BOS appears inside the document".to_string(),
            );
        }
        for (offset, &token) in tokens.iter().enumerate() {
            if u32::from(token) >= VOCAB_SIZE || u32::from(token) == UNK_ID {
                ledger.hit(
                    "document_vocabulary",
                    document,
                    format!("token {} at document offset {offset}", token),
                );
            }
        }
        for (offset, &value) in mask.iter().enumerate() {
            if value > 1 {
                ledger.hit(
                    "document_mask_binary",
                    document,
                    format!("mask byte {value} at document offset {offset}"),
                );
            }
        }
        // Every user marker must be the exact literal and carry mask 0, in every turn.
        if tokens.len() < 4
            || tokens[1..4] != USER_MARKER.map(|id| id as u16)
            || mask[1..4].iter().any(|&value| value != 0)
        {
            ledger.hit(
                "user_marker_exact",
                document,
                format!(
                    "tokens 1..4 are {:?} with mask {:?}",
                    &tokens[..tokens.len().min(4)],
                    &mask[..mask.len().min(4)]
                ),
            );
        }
        for cursor in 0..tokens.len().saturating_sub(USER_MARKER.len() - 1) {
            if tokens[cursor..cursor + USER_MARKER.len()] == USER_MARKER.map(|id| id as u16)
                && mask[cursor..cursor + USER_MARKER.len()]
                    .iter()
                    .any(|&value| value != 0)
            {
                ledger.hit(
                    "user_marker_exact",
                    document,
                    format!("a User: marker at offset {cursor} carries a masked token"),
                );
                break;
            }
        }

        // Every assistant marker opens exactly one contiguous masked run that closes at its own
        // EOS, with an unmasked token (or the document end) after it.
        let assistant = ASSISTANT_MARKER.map(|id| id as u16);
        for marker in 0..tokens.len().saturating_sub(assistant.len() - 1) {
            if tokens[marker..marker + assistant.len()] != assistant {
                continue;
            }
            let run_start = marker + assistant.len();
            if run_start >= tokens.len() || mask[run_start] != 1 {
                ledger.hit(
                    "response_single_run",
                    document,
                    format!("assistant marker at offset {marker} opens no masked response"),
                );
                continue;
            }
            let mut run_end = run_start;
            while run_end < mask.len() && mask[run_end] == 1 {
                run_end += 1;
            }
            let closes = (run_start..run_end)
                .find(|&position| u32::from(tokens[position]) == EOS_ID)
                .is_some_and(|eos| {
                    eos + 1 == run_end && (eos + 1 == tokens.len() || mask[eos + 1] == 0)
                });
            if !closes {
                ledger.hit(
                    "response_single_run",
                    document,
                    format!(
                        "the masked stretch after the marker at offset {marker} does not close at \
                         its own EOS followed by an unmasked token"
                    ),
                );
            }
        }

        // Per-run clauses.
        for (run_start, run_end) in mask_runs(mask) {
            if run_start >= run_end {
                ledger.hit(
                    "response_nonempty",
                    document,
                    "empty response run".to_string(),
                );
                continue;
            }
            if run_start <= ASSISTANT_MARKER.len() {
                ledger.hit(
                    "response_marker_present",
                    document,
                    format!("response run starts at document offset {run_start}"),
                );
                continue;
            }
            let marker_start = run_start - ASSISTANT_MARKER.len();
            if mask[marker_start..run_start].iter().any(|&value| value != 0) {
                ledger.hit(
                    "response_marker_unmasked",
                    document,
                    format!("mask {:?} over the marker", &mask[marker_start..run_start]),
                );
            }
            if tokens[marker_start..run_start] != ASSISTANT_MARKER.map(|id| id as u16) {
                ledger.hit(
                    "response_marker_exact",
                    document,
                    format!(
                        "tokens {:?} before the response run",
                        &tokens[marker_start..run_start]
                    ),
                );
            }
            if u32::from(tokens[run_end - 1]) != EOS_ID || mask[run_end - 1] != 1 {
                ledger.hit(
                    "response_terminal_eos",
                    document,
                    format!(
                        "last response token is {} with mask {}",
                        tokens[run_end - 1],
                        mask[run_end - 1]
                    ),
                );
            }
            if tokens[run_start..run_end - 1]
                .iter()
                .any(|&token| u32::from(token) == EOS_ID)
            {
                ledger.hit(
                    "response_no_interior_eos",
                    document,
                    "an EOS appears inside the response run".to_string(),
                );
            }
        }
    }
    ledger
}

/// Every assistant turn re-read as its own single-turn document, under the same sixteen clause
/// names with their literal single-turn meaning (for a single-run document the multi-turn
/// generalisations coincide with the literal checks).
fn validate_turn_units(documents: &[TwoTurnDoc]) -> Ledger {
    let mut stream: Vec<u16> = Vec::new();
    let mut mask: Vec<u8> = Vec::new();
    for document in documents {
        let runs = mask_runs(&document.mask);
        let mut cursor = 1usize; // after BOS
        for (run_start, run_end) in runs {
            let marker_start = run_start - ASSISTANT_MARKER.len();
            push(&mut stream, &mut mask, &[BOS_ID as u16], 0);
            push(
                &mut stream,
                &mut mask,
                &document.tokens[cursor..marker_start],
                0,
            );
            push(
                &mut stream,
                &mut mask,
                &document.tokens[marker_start..run_start],
                0,
            );
            push(
                &mut stream,
                &mut mask,
                &document.tokens[run_start..run_end],
                1,
            );
            cursor = run_end;
            if cursor < document.tokens.len()
                && document.tokens[cursor] == SEPARATOR_ID
                && document.mask[cursor] == 0
            {
                cursor += 1;
            }
        }
    }
    validate_documents(&stream, &mask)
}

/// The literal single-turn readings of the two generalised clauses, over the whole two-turn
/// documents: documents with more than one masked run, and documents with any interior EOS.
fn literal_single_turn_counts(stream: &[u16], mask: &[u8]) -> (usize, usize) {
    let starts = document_starts(stream);
    let mut multi_run = 0usize;
    let mut interior_eos = 0usize;
    for document in 0..starts.len() - 1 {
        let (start, end) = (starts[document], starts[document + 1]);
        let mask = &mask[start..end];
        let tokens = &stream[start..end];
        if mask_runs(mask).len() != 1 {
            multi_run += 1;
        }
        if tokens.len() > 2
            && tokens[1..tokens.len() - 1]
                .iter()
                .any(|&token| u32::from(token) == EOS_ID)
        {
            interior_eos += 1;
        }
    }
    (multi_run, interior_eos)
}

/// (b) Prove the validator bites: corrupt documents in memory only, show the offending document
/// index and clause, and re-run the clean validation to show the store on disk is untouched.
fn corruption_test(stream: &[u16], mask: &[u8]) {
    let starts = document_starts(stream);
    let document = starts.len() - 3; // a fixed, reported index
    let (start, end) = (starts[document], starts[document + 1]);
    println!(
        "corruption test (in memory only; the store on disk is never written): document {document} \
         spans [{start},{end})"
    );

    // Case 1: leave the document's terminal EOS unmasked.
    let mut corrupted = mask.to_vec();
    corrupted[end - 1] = 0;
    report_corruption("case 1 (terminal EOS of the document left unmasked)", stream, &corrupted);

    // Case 2: mask the separator that opens the second turn, merging the two response runs.
    let runs = mask_runs(&mask[start..end]);
    match runs.get(1) {
        Some(&(second_run_start, _)) => {
            let marker_start = start + second_run_start - ASSISTANT_MARKER.len();
            let separator = marker_start - 1;
            let mut corrupted = mask.to_vec();
            corrupted[separator] = 1;
            report_corruption(
                &format!("case 2 (turn separator at {separator} masked)"),
                stream,
                &corrupted,
            );
        }
        None => eprintln!("  corruption case 2 found no second response run to merge"),
    }

    // Restore: the same validator over the untouched bytes.
    let restored = validate_documents(stream, mask);
    println!(
        "  restored (original bytes): {} offenders over {} clauses",
        restored.offenders(),
        restored.counts.len()
    );
    if restored.offenders() != 0 {
        eprintln!("  the restored store did not validate clean");
    }
}

fn report_corruption(title: &str, stream: &[u16], corrupted: &[u8]) {
    let ledger = validate_documents(stream, corrupted);
    println!("  {title}: {} offenders", ledger.offenders());
    for (clause, count) in &ledger.counts {
        if *count > 0 {
            println!("    clause {clause}: {count}");
        }
    }
    for (document, clause, detail) in ledger.offenders.iter().take(4) {
        println!("    document {document} clause {clause}: {detail}");
    }
    if ledger.offenders() == 0 {
        eprintln!("  {title} was NOT detected; the validator does not bite");
    }
    // The same corrupted bytes through the training path's own episode index.
    let contract = EpisodeContract {
        context: EPISODE_CONTEXT,
        vocab_size: VOCAB_SIZE as usize,
        bos_id: BOS_ID,
        eos_id: EOS_ID,
        unk_id: UNK_ID,
        padding_id: PADDING_ID,
        assistant_marker_ids: ASSISTANT_MARKER.to_vec(),
    };
    let sources = [SourceSpan {
        label: OUTPUT_LABEL.to_string(),
        start: 0,
        end: stream.len(),
    }];
    match EpisodeIndex::new(stream, corrupted, contract, &sources) {
        Ok(index) => println!(
            "    training-path EpisodeIndex admitted the corrupted store: documents {}, runs {}, \
             excluded {}",
            index.population().documents,
            index.population().response_runs,
            index.population().excluded_over_context
        ),
        Err(error) => println!("    training-path EpisodeIndex rejected it: {error}"),
    }
}

fn report_ledger(title: &str, ledger: &Ledger) {
    println!(
        "validation ({title}): {} clauses, {} offenders",
        ledger.counts.len(),
        ledger.offenders()
    );
    for (clause, count) in &ledger.counts {
        println!("  clause {clause}: {count}");
    }
    if ledger.offenders() != 0 {
        for (document, clause, detail) in ledger.offenders.iter().take(MAX_REPORTED_OFFENDERS) {
            eprintln!("  document {document} clause {clause}: {detail}");
        }
    }
}

fn write_store(
    directory: &Path,
    stream: &[u16],
    mask: &[u8],
    args: &Args,
    sources: &[SourceDoc],
    tokenizer_sha256: &str,
) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|e| format!("create {}: {e}", directory.display()))?;
    let tokens_path = directory.join("tokens.u16");
    CorpusWriter::write_file(&tokens_path, VOCAB_SIZE, stream)
        .map_err(|e| format!("write {}: {e}", tokens_path.display()))?;
    let mask_path = directory.join("response_mask.u8");
    fs::write(&mask_path, mask).map_err(|e| format!("write {}: {e}", mask_path.display()))?;
    let source_tokens = args.source.join("tokens.u16");
    let source_mask = args.source.join("response_mask.u8");
    let source_manifest = args.source.join("manifest.json");
    let manifest = serde_json::json!({
        "dialogue_protocol": "uor-r4.literal-role-dialogue/2",
        "drops": {"special_token_occurrences": 0},
        "files": [{
            "label": OUTPUT_LABEL,
            "tokens": stream.len(),
            "special_token_occurrences": 0,
        }],
        "mask_bytes": mask.len(),
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating \
                      <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, \
                      and an unmasked document-terminal <|eos|>.",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "mask_sha256": uor_r4_training::sha256_bytes(mask),
        "response_tokens": mask.iter().filter(|&&value| value == 1).count(),
        "schema": "uor-r4-demand-store-2turn/v1",
        "source_store": {
            "documents": sources.len(),
            "manifest": source_manifest.display().to_string(),
            "manifest_sha256": uor_r4_training::sha256_file(&source_manifest)
                .map_err(|e| e.to_string())?,
            "response_mask_u8": source_mask.display().to_string(),
            "response_mask_u8_sha256": uor_r4_training::sha256_file(&source_mask)
                .map_err(|e| e.to_string())?,
            "tokens_u16": source_tokens.display().to_string(),
            "tokens_u16_sha256": uor_r4_training::sha256_file(&source_tokens)
                .map_err(|e| e.to_string())?,
        },
        "template_rule": "<|bos|> then turns joined by a single '\\n' separator (placed before \
                          every turn after the first); a turn is '<marker><content>' with markers \
                          'System: ', 'User: ', 'Assistant: '; every assistant turn ends with \
                          <|eos|>; a document-terminal <|eos|> is appended only when the final \
                          emitted turn is not assistant.",
        "tokenizer": {
            "bos_id": BOS_ID,
            "eos_id": EOS_ID,
            "path": args.tokenizer.display().to_string(),
            "sha256": tokenizer_sha256,
            "unk_id": UNK_ID,
            "vocab_size": VOCAB_SIZE,
        },
        "tokens": stream.len(),
        "tokens_bytes": CORPUS_HEADER_SIZE + stream.len() * 2,
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "generator": "crates/uor-r4-training/examples/demand-store-2turn.rs",
        "seed": format!("0x{:016x}", args.seed),
    });
    let manifest_path = directory.join("manifest.json");
    let bytes =
        serde_json::to_vec_pretty(&manifest).map_err(|e| format!("serialize manifest: {e}"))?;
    fs::write(&manifest_path, bytes)
        .map_err(|e| format!("write {}: {e}", manifest_path.display()))?;
    Ok(())
}

/// Re-read the emitted bytes and rebuild the real episode index over them, so the store is
/// accepted by the training path's own reader rather than by the validator above.
fn verify_store(
    directory: &Path,
    stream: &[u16],
    mask: &[u8],
) -> Result<uor_r4_training::dialogue_episodes::EpisodePopulation, String> {
    let tokens_path = directory.join("tokens.u16");
    let mask_path = directory.join("response_mask.u8");
    let manifest_path = directory.join("manifest.json");
    let reader = MmapCorpusReader::open(&tokens_path)
        .map_err(|e| format!("reopen {}: {e}", tokens_path.display()))?;
    if reader.total_tokens() != stream.len() || reader.vocab_size() != VOCAB_SIZE {
        return Err(format!(
            "{} declares {} tokens at vocabulary {}; expected {} at {VOCAB_SIZE}",
            tokens_path.display(),
            reader.total_tokens(),
            reader.vocab_size(),
            stream.len()
        ));
    }
    if reader.as_slice() != stream {
        return Err(format!(
            "{} payload differs from the generated stream",
            tokens_path.display()
        ));
    }
    let stored = fs::read(&mask_path).map_err(|e| format!("read {}: {e}", mask_path.display()))?;
    if stored != mask {
        return Err(format!(
            "{} differs from the generated mask",
            mask_path.display()
        ));
    }
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|e| format!("read {}: {e}", manifest_path.display()))?,
    )
    .map_err(|e| format!("parse {}: {e}", manifest_path.display()))?;
    let files = manifest["files"]
        .as_array()
        .ok_or_else(|| "manifest has no files array".to_string())?;
    if manifest["drops"]["special_token_occurrences"] != 0
        || manifest["mask_bytes"].as_u64() != Some(mask.len() as u64)
        || manifest["mask_sha256"].as_str() != Some(uor_r4_training::sha256_bytes(mask).as_str())
        || files.len() != 1
        || files[0]["label"] != OUTPUT_LABEL
        || files[0]["tokens"].as_u64() != Some(stream.len() as u64)
        || files[0]["special_token_occurrences"] != 0
    {
        return Err("manifest fields do not match the written store".to_string());
    }
    let contract = EpisodeContract {
        context: EPISODE_CONTEXT,
        vocab_size: VOCAB_SIZE as usize,
        bos_id: BOS_ID,
        eos_id: EOS_ID,
        unk_id: UNK_ID,
        padding_id: PADDING_ID,
        assistant_marker_ids: ASSISTANT_MARKER.to_vec(),
    };
    let split = DialogueSplit::load(&tokens_path, &mask_path, &manifest_path)
        .map_err(|e| format!("stack dialogue manifest reader rejected the store: {e}"))?;
    if split.vocab_size() != VOCAB_SIZE as usize {
        return Err(format!(
            "loaded vocabulary {} is not {VOCAB_SIZE}",
            split.vocab_size()
        ));
    }
    if split.source_range(OUTPUT_LABEL) != Some(0..stream.len()) {
        return Err(format!(
            "no single source labelled {OUTPUT_LABEL} covers the store"
        ));
    }
    let index = split
        .index(contract)
        .map_err(|e| format!("the training-time episode index rejected the emitted store: {e}"))?;
    let population = index.population().clone();
    if index.episodes().len() != population.eligible_responses {
        return Err("the admitted episode count differs from the population".to_string());
    }
    if population.excluded_over_context != 0 {
        return Err(format!(
            "the episode index excluded {} responses over context",
            population.excluded_over_context
        ));
    }
    Ok(population)
}
