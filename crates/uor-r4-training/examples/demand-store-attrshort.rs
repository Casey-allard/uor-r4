//! Build an ATTRIBUTE-EXTRACTION demand store with SHORT PREMISES.
//!
//! ```text
//! demand-store-attrshort <output-dir> [--tokenizer <tokenizer.json>] [--seed N] [--corrupt-test]
//!                        [--samples N]
//! ```
//!
//! The two-turn attribute store (`demand-store-attr.rs`) proved that a store can carry perfect
//! dependency structure and still cost form compliance: its turn-1 answers were LONG authored
//! sentences (2-31 tokens) and it contributed 53,671 response tokens, and the mixture's compliance
//! fell from ten of ten to two of ten. This tool rebuilds the same dependency idea with SHORT
//! premises: turn 1 is a short list, a symbol, a one-clause fact or a bare number, and its answer is
//! at most `MAX_ANSWER1_TOKENS` (8) response-content tokens with an emitted mean at or below
//! `TARGET_ANSWER1_MEAN` (5). Every second turn asks for a SPECIFIC ATTRIBUTE of turn 1 (its count,
//! its second item, its symbol, its colour, a number derived from it, the polarity of its answer),
//! so copying turn 1's answer verbatim is wrong for every document, while the response mass stays
//! far below the 31,084 response tokens of the two-turn store that produced ten of ten form
//! compliance.
//!
//! The confound this store is built to separate: the attribute store's collapse could mean
//! "attribute extraction is intrinsically hard" or "long authored answers re-taught the
//! long-answer mode". This store carries the same transformation structure with short premises and
//! short answers, so a mixture that keeps compliance with it separates the two readings; a mixture
//! that loses compliance with it does not.
//!
//! The single-turn demand store (`demand-store.rs`) teaches answer-form compliance, and the
//! two-turn store (`demand-store-2turn.rs`) adds a second turn — but for 1,294 of its 2,860
//! documents the second turn's answer is TOKEN-IDENTICAL to the first turn's, so a "copy the
//! previous response" policy satisfies them. This tool emits the complementary store: every
//! second turn demands a short form whose answer is a TRANSFORMATION of turn 1's answer (a count,
//! an ordinal, a derived number, a polarity) or a PART of it (one named word out of a sentence), so
//! copying turn 1's answer verbatim is wrong for every document.
//!
//! ```text
//! BOS  User:  <turn 1 question>  \n  Assistant:  <turn 1 answer>  EOS
//!      \n  User:  <turn 2 demand>  \n  Assistant:  <turn 2 answer>  EOS
//! mask 0    mask 0                0     mask 0         1   1      0
//!        0       0                0       0            1   1
//! ```
//!
//! The framing is the prepared chat corpus's multi-turn convention (`uor-r4.literal-role-dialogue/2`,
//! chat manifest `template_rule`): BOS, then turns joined by a single `\n` separator placed before
//! every turn after the first; a turn is `<marker><content>` with `User: `/`Assistant: ` markers;
//! every assistant turn ends with `<|eos|>`; a document-terminal `<|eos|>` is appended only when
//! the final emitted turn is not assistant. That convention was checked against the real chat
//! corpus before `demand-store-2turn.rs` was written: the gap between consecutive response runs is
//! `\n`(201) `User:`(55,2728,28) … `\n`(201) `Assistant:`(35,560,652,714,28) in 55,839 of 55,840
//! cases, and 73,370 of 73,625 documents end at the final assistant turn's masked EOS with no
//! extra document EOS. The mask rule is the chat manifest's: 1 on each assistant turn's response
//! content and its terminating `<|eos|>`, 0 on BOS, role markers, turn separators and all user
//! content.
//!
//! Two of the single-turn generator's sixteen clauses are per-document there and cannot hold
//! literally for a document with two assistant turns; this tool checks their multi-turn
//! generalisation and prints the literal single-turn counts as an annex:
//!
//! - `document_no_interior_eos` -> no *unmasked* EOS strictly inside a document (the training
//!   path's own rule in `dialogue_episodes.rs`); a completed masked turn EOS is expected interior.
//! - `response_single_run` -> each assistant marker opens exactly one contiguous masked run that
//!   closes at its own EOS, with an unmasked token (or the document end) after it; a two-turn
//!   document holds one run per turn.
//!
//! Every other clause is the source tools' clause, applied per document or per response run. The
//! tool additionally re-checks every assistant turn as a literal single-turn document (the
//! unchanged single-turn clause set) so both readings are reported. Nothing is written unless all
//! clause counters are zero and the transformation census below passes; after writing, the store
//! is re-read through `DialogueSplit::load` and the real `EpisodeIndex`, so acceptance is the
//! training path's own check. `--corrupt-test` corrupts one document in memory (never on disk),
//! shows the offending document index and clause, restores, and is repeated over the bytes read
//! back from the emitted store.
//!
//! Families (every emitted document belongs to exactly one; every turn-1 answer is short):
//!
//! - `count`:     t1 `Metals: iron, copper, zinc.` -> `Noted.`,
//!                t2 `Answer with digits only. How many items did I list?` -> `3.`
//! - `ordinal`:   t1 `Colours: red, orange, yellow.` -> `I see.`,
//!                t2 `One word only. What was the second colour?` -> `orange.`
//! - `name`:      t1 `Symbol: Ar.` -> `Argon.`,
//!                t2 `Give a one-word answer. Which symbol did I give?` -> `Ar.`
//! - `attribute`: t1 `The rose is red.` -> `Noted.`,
//!                t2 `One word only. What colour did I say the rose was?` -> `red.`
//! - `number`:    t1 `The number is 20.` -> `Twenty.`,
//!                t2 `Answer with digits only. What is that number minus one?` -> `19.`
//! - `polarity`:  t1 `Is the sun a planet?` -> `No.`,
//!                t2 `Reply with yes or no. Was my last answer no?` -> `Yes.`
//!
//! Every turn-2 answer is a genuine attribute or transformation of turn 1, never a repeat of it:
//! the census refuses to write unless the transformation fraction is exactly 1.0, and the length
//! gate refuses to write unless the emitted turn-1 answers have mean <= 5 and max <= 8 tokens.

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
/// This store's source label in the manifest.
const OUTPUT_LABEL: &str = "demand-attrshort";
/// This tool's document-order seed; `--seed` overrides it.
const DEFAULT_SEED: u64 = 0x6174_7472_7368_7274; // "attrshrt"
/// The hard ceiling on a turn-1 answer's response-content tokens (its EOS excluded).
const MAX_ANSWER1_TOKENS: usize = 8;
/// The emitted mean turn-1 answer length must not exceed this.
const TARGET_ANSWER1_MEAN: f64 = 5.0;
/// The hard ceiling on a turn-2 answer's response-content tokens (its EOS excluded).
const MAX_ANSWER2_TOKENS: usize = 4;
/// Candidate preference boundary: a turn-1 answer at or below it is selected first, so the emitted
/// mean lands well inside the target instead of at the ceiling.
const PREFERRED_ANSWER1_TOKENS: usize = 5;
/// The compliant configuration range the store must stay inside (deliberately SMALL).
const MIN_DOCUMENTS: usize = 900;
const MAX_DOCUMENTS: usize = 1200;
/// Response tokens of the two-turn store that produced ten of ten form compliance; this store's
/// response mass must stay materially below it.
const COMPLIANT_STORE_RESPONSE_TOKENS: usize = 31_084;
/// How many offending documents the refusal prints before summarizing.
const MAX_REPORTED_OFFENDERS: usize = 20;
/// The demanded floor on the fraction of documents whose second answer is not a verbatim copy of
/// the first answer's token sequence. This store is built so the fraction is exactly 1.0.
const MIN_TRANSFORMATION_FRACTION: f64 = 1.0;

/// Every clause the store must satisfy. The names mirror the demand stores' generators, which
/// mirror the training-time checks in `dialogue_episodes.rs`; the counter for each must end at
/// zero.
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

/// One authored two-turn document before tokenization.
#[derive(Clone)]
struct Spec {
    family: &'static str,
    question1: String,
    answer1: String,
    question2: String,
    answer2: String,
}

/// One emitted two-turn document with the provenance the census needs.
struct Doc {
    tokens: Vec<u16>,
    mask: Vec<u8>,
    family: &'static str,
    /// The response content of turn 1 without its EOS, as emitted.
    answer1_ids: Vec<u16>,
    /// The response content of turn 2 without its EOS, as emitted.
    answer2_ids: Vec<u16>,
    /// The first token of turn 2's response run; the prefix a copy policy could read is `[0, ..)`.
    response2_start: usize,
    answer1_text: String,
    answer2_text: String,
}

/// (b) The answer-length gate's measurements over the emitted documents. Lengths are
/// response-CONTENT tokens: every turn's terminating EOS is excluded, as is every unmasked token.
struct LengthStats {
    documents: usize,
    mean1: f64,
    max1: usize,
    mean2: f64,
    max2: usize,
    total1: usize,
    total2: usize,
    histogram1: BTreeMap<usize, usize>,
    histogram2: BTreeMap<usize, usize>,
}

impl LengthStats {
    fn new(documents: &[Doc]) -> Self {
        let mut histogram1: BTreeMap<usize, usize> = BTreeMap::new();
        let mut histogram2: BTreeMap<usize, usize> = BTreeMap::new();
        let mut total1 = 0usize;
        let mut total2 = 0usize;
        let mut max1 = 0usize;
        let mut max2 = 0usize;
        for document in documents {
            let length1 = document.answer1_ids.len();
            let length2 = document.answer2_ids.len();
            total1 += length1;
            total2 += length2;
            max1 = max1.max(length1);
            max2 = max2.max(length2);
            *histogram1.entry(length1).or_default() += 1;
            *histogram2.entry(length2).or_default() += 1;
        }
        let count = documents.len().max(1);
        Self {
            documents: documents.len(),
            mean1: total1 as f64 / count as f64,
            max1,
            mean2: total2 as f64 / count as f64,
            max2,
            total1,
            total2,
            histogram1,
            histogram2,
        }
    }

    fn report(&self) {
        println!(
            "answer lengths (b, response-content tokens, EOS excluded) over {} documents: turn 1 \
             mean {:.4} max {} (histogram length:count {self_hist1:?}); turn 2 mean {:.4} max {} \
             (histogram {self_hist2:?}); total content tokens {}/{}",
            self.documents,
            self.mean1,
            self.max1,
            self.mean2,
            self.max2,
            self.total1,
            self.total2,
            self_hist1 = self.histogram1,
            self_hist2 = self.histogram2,
        );
    }
}

/// Per-family transformation census.
#[derive(Clone, Copy, Default)]
struct FamilyRelation {
    documents: usize,
    equal: usize,
    span_in_answer1: usize,
    span_in_prefix: usize,
}

/// The transformation census over every emitted document.
struct RelationStats {
    documents: usize,
    /// Second answer token-for-token identical to the first answer: a copy policy would pass.
    equal: usize,
    /// Second answer is a contiguous span of the first answer: a span-copy policy would pass.
    span_in_answer1: usize,
    /// Second answer is a contiguous span anywhere before turn 2's response.
    span_in_prefix: usize,
    per_family: BTreeMap<&'static str, FamilyRelation>,
}

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
    output: PathBuf,
    tokenizer: PathBuf,
    seed: u64,
    corrupt_test: bool,
    samples: usize,
}

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&argv) {
        eprintln!("demand-store-attrshort: {error}");
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

    let (specs, family_report) = select_specs(args.seed, &tokenizer)?;
    let documents = build_documents(&specs, &tokenizer)?;
    let mut stream: Vec<u16> = Vec::new();
    let mut mask: Vec<u8> = Vec::new();
    for document in &documents {
        stream.extend_from_slice(&document.tokens);
        mask.extend_from_slice(&document.mask);
    }

    let response_tokens = mask.iter().filter(|&&value| value == 1).count();
    println!(
        "emitted: {} two-turn documents, {} tokens, {} response tokens, {} response runs",
        documents.len(),
        stream.len(),
        response_tokens,
        2 * documents.len()
    );
    for (family, (enumerated, emitted)) in &family_report {
        println!("  family {family}: {emitted} documents from {enumerated} candidates");
    }
    if !(MIN_DOCUMENTS..=MAX_DOCUMENTS).contains(&documents.len()) {
        return Err(format!(
            "refusing to write: {} documents is outside the {MIN_DOCUMENTS}-{MAX_DOCUMENTS} window",
            documents.len()
        ));
    }

    // (b) The length gate: what the store teaches as answers must stay short.
    let lengths = LengthStats::new(&documents);
    lengths.report();
    if lengths.max1 > MAX_ANSWER1_TOKENS {
        return Err(format!(
            "refusing to write: a turn-1 answer holds {} tokens, over the {MAX_ANSWER1_TOKENS} \
             ceiling",
            lengths.max1
        ));
    }
    if lengths.max2 > MAX_ANSWER2_TOKENS {
        return Err(format!(
            "refusing to write: a turn-2 answer holds {} tokens, over the {MAX_ANSWER2_TOKENS} \
             ceiling",
            lengths.max2
        ));
    }
    if lengths.mean1 > TARGET_ANSWER1_MEAN {
        return Err(format!(
            "refusing to write: the emitted mean turn-1 answer is {:.4} tokens, over the target \
             {TARGET_ANSWER1_MEAN:.2}",
            lengths.mean1
        ));
    }
    if response_tokens >= COMPLIANT_STORE_RESPONSE_TOKENS {
        return Err(format!(
            "refusing to write: {response_tokens} response tokens are not below the \
             {COMPLIANT_STORE_RESPONSE_TOKENS} of the compliant two-turn store"
        ));
    }
    let mass_ratio = response_tokens as f64 / COMPLIANT_STORE_RESPONSE_TOKENS as f64;
    println!(
        "response mass (c vs the compliant store): {response_tokens} response tokens, {:.4} of \
         {COMPLIANT_STORE_RESPONSE_TOKENS} ({:.2}% below), mean turn-1 answer {:.4} tokens (max \
         {}), mean turn-2 answer {:.4} tokens (max {}) over {} documents",
        mass_ratio,
        (1.0 - mass_ratio) * 100.0,
        lengths.mean1,
        lengths.max1,
        lengths.mean2,
        lengths.max2,
        documents.len()
    );

    // (b) The transformation census: copying turn 1's answer verbatim must fail.
    let relation = relation_census(&documents);
    report_relation(&relation);
    if relation.equal != 0 {
        return Err(format!(
            "refusing to write: {} of {} documents repeat turn 1's answer token-for-token",
            relation.equal, relation.documents
        ));
    }
    let fraction = (relation.documents - relation.equal) as f64 / relation.documents as f64;
    if fraction < MIN_TRANSFORMATION_FRACTION {
        return Err(format!(
            "refusing to write: transformation fraction {fraction:.6} is below \
             {MIN_TRANSFORMATION_FRACTION:.2}"
        ));
    }

    // (a) the sixteen clauses over the emitted bytes, multi-turn reading.
    let ledger = validate_documents(&stream, &mask);
    report_ledger("multi-turn document reading", &ledger);
    // (a') the same sixteen clause names with their literal single-turn reading, applied to every
    // assistant turn as its own single-turn document.
    let unit_ledger = validate_turn_units(&documents);
    report_ledger(
        "literal single-turn reading, one unit per assistant turn",
        &unit_ledger,
    );
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

    // (c) the validator must bite: corrupt one document in memory and show it is caught.
    let corrupt_index = documents.len() / 2;
    if args.corrupt_test {
        corruption_test(
            "in-memory store before writing",
            &stream,
            &mask,
            corrupt_index,
        )?;
    }

    // (d) write, then re-read through the training path's own reader.
    write_store_files(&args.output, &stream, &mask)?;
    write_manifest(
        &args.output,
        &stream,
        &mask,
        &args,
        &tokenizer_sha256,
        &family_report,
        &relation,
        &lengths,
        fraction,
        None,
    )?;
    let population = verify_store(&args.output, &stream, &mask)?;
    if population.excluded_over_context != 0 {
        return Err(format!(
            "the episode index excluded {} responses over context; the store is unusable",
            population.excluded_over_context
        ));
    }
    println!(
        "reader (DialogueSplit::load + EpisodeIndex, context {EPISODE_CONTEXT}): documents {}, \
         response runs {}, response tokens {}, ADMITTED {}, admitted response tokens {}, EXCLUDED \
         over context {}, identical prefixes {}",
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
        mask.iter().filter(|&&value| value == 1).count() as f64 / stream.len() as f64,
        stream.len() as f64 / documents.len() as f64,
        population.response_runs as f64 / documents.len() as f64,
    );

    // Seal the manifest with the reader's own verified counts, then re-load it so the file that is
    // reported is the file that was accepted.
    write_manifest(
        &args.output,
        &stream,
        &mask,
        &args,
        &tokenizer_sha256,
        &family_report,
        &relation,
        &lengths,
        fraction,
        Some(&population),
    )?;
    let final_population = verify_store(&args.output, &stream, &mask)?;
    if final_population != population {
        return Err("the re-sealed manifest changed the reader's population".to_string());
    }
    if args.corrupt_test {
        // The same corruption test over the bytes as they are on disk: the emitted artifact is
        // what the validator and the reader actually accept.
        let reader = MmapCorpusReader::open(args.output.join("tokens.u16"))
            .map_err(|e| format!("reopen emitted tokens: {e}"))?;
        let stored_mask = fs::read(args.output.join("response_mask.u8"))
            .map_err(|e| format!("read emitted mask: {e}"))?;
        corruption_test(
            "emitted store re-read from disk",
            reader.as_slice(),
            &stored_mask,
            corrupt_index,
        )?;
    }

    // (e) the first three documents, decoded with their ids and masks.
    for (index, document) in documents.iter().take(3).enumerate() {
        let ids: Vec<u32> = document
            .tokens
            .iter()
            .map(|&token| u32::from(token))
            .collect();
        println!(
            "document {index} (family {}, turn-1 answer {:?}, turn-2 answer {:?}): {}",
            document.family,
            document.answer1_text,
            document.answer2_text,
            tokenizer.decode(&ids).replace('\n', " \\n ")
        );
        println!("  tokens: {ids:?}");
        println!("  mask:   {:?}", document.mask);
        println!(
            "  turn-1 answer ids {:?} (mask 1), turn-2 answer ids {:?} (mask 1)",
            document.answer1_ids, document.answer2_ids
        );
    }
    if args.samples > 0 {
        let mut shown: BTreeMap<&'static str, usize> = BTreeMap::new();
        println!("samples ({} per family):", args.samples);
        for document in &documents {
            let seen = shown.entry(document.family).or_default();
            if *seen >= args.samples {
                continue;
            }
            *seen += 1;
            let ids: Vec<u32> = document
                .tokens
                .iter()
                .map(|&token| u32::from(token))
                .collect();
            println!(
                "  [{}] turn-1 answer {:?} -> turn-2 answer {:?}\n    {}",
                document.family,
                document.answer1_text,
                document.answer2_text,
                tokenizer.decode(&ids).replace('\n', " \\n ")
            );
        }
    }

    let tokens_path = args.output.join("tokens.u16");
    let mask_path = args.output.join("response_mask.u8");
    let manifest_path = args.output.join("manifest.json");
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
        "  manifest.json sha256 {}",
        uor_r4_training::sha256_file(&manifest_path).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
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
                    "usage: demand-store-attrshort <output-dir> [--tokenizer <tokenizer.json>] \
                     [--seed N] [--corrupt-test] [--samples N]"
                );
                std::process::exit(0);
            }
            other if other.starts_with("--") => return Err(format!("unknown flag {other}")),
            other => {
                if output.is_none() {
                    output = Some(PathBuf::from(other));
                } else {
                    return Err(format!("unexpected extra argument {other}"));
                }
            }
        }
        index += 1;
    }
    Ok(Args {
        output: output.ok_or_else(usage)?,
        tokenizer: tokenizer.unwrap_or_else(default_tokenizer),
        seed,
        corrupt_test,
        samples,
    })
}

fn usage() -> String {
    "usage: demand-store-attrshort <output-dir> [--tokenizer <tokenizer.json>] [--seed N] \
     [--corrupt-test] [--samples N]"
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

/// `a, b, c` — the comma-joined short list the count and ordinal families are built from.
fn comma_list(items: &[&str]) -> String {
    items.join(", ")
}

fn fill(template: &str, key: &str, value: &str) -> String {
    template.replace(key, value)
}

fn quota_for(family: &str) -> Result<usize, String> {
    FAMILY_QUOTAS
        .iter()
        .find(|entry| entry.0 == family)
        .map(|entry| entry.1)
        .ok_or_else(|| format!("family {family} has no declared quota"))
}
/// Per-family emission quotas: each family's enumerated candidate set is filtered to the length
/// bounds, shuffled with the seed and truncated to its quota, so every family is present in the
/// declared proportion and no exact duplicate document can appear. The total (1,100) is inside the
/// 900-1,200 window the compliant configurations used.
const FAMILY_QUOTAS: &[(&str, usize)] = &[
    ("count", 240),
    ("ordinal", 220),
    ("name", 180),
    ("attribute", 220),
    ("number", 160),
    ("polarity", 80),
];

/// `(category, singular, items)`: the SHORT item lists the count and ordinal families read a number
/// or a position out of. Turn 1 acknowledges them in one short form and turn 2 demands the count or
/// the item, so the demanded content lives only in turn 1's user turn and never in its answer.
const ITEM_LISTS: &[(&str, &str, &[&str])] = &[
    ("Metals", "metal", &["iron", "copper", "zinc"]),
    ("Colours", "colour", &["red", "orange", "yellow"]),
    ("Shapes", "shape", &["circle", "square", "oval"]),
    ("Tools", "tool", &["hammer", "chisel", "plane"]),
    ("Rivers", "river", &["Nile", "Rhine", "Volga"]),
    ("Birds", "bird", &["wren", "robin", "finch"]),
    ("Planets", "planet", &["Mars", "Venus", "Saturn"]),
    ("Languages", "language", &["Latin", "Greek", "Arabic"]),
    ("Trees", "tree", &["oak", "pine", "birch"]),
    ("Instruments", "instrument", &["flute", "violin", "harp"]),
    ("Fruits", "fruit", &["apple", "pear", "plum", "fig"]),
    ("Animals", "animal", &["cat", "dog", "fox", "owl"]),
    ("Cities", "city", &["Paris", "Rome", "Oslo", "Cairo"]),
    ("Flowers", "flower", &["rose", "tulip", "daisy", "lily"]),
    ("Stones", "stone", &["flint", "slate", "marble", "chalk"]),
    ("Drinks", "drink", &["water", "juice", "tea", "milk"]),
];

/// Short turn-1 premise templates for the count family; `{cat}` is the capitalized category and
/// `{list}` the comma-joined items.
const COUNT_ASKS: &[&str] = &[
    "{cat}: {list}.",
    "Items: {list}.",
    "Here: {list}.",
    "{cat} list: {list}.",
    "My list: {list}.",
];

/// Turn-2 demands for the count family; `{plural}` is the lower-case category. Turn 1's answer is
/// the echoed list, so the count is never a copy of it.
const COUNT_DEMANDS: &[&str] = &[
    "Answer with digits only. How many items did I list?",
    "Digits only. How many {plural} did I name?",
    "Answer with digits only. How many items were in my list?",
    "Reply with a number only. How many {plural} were there?",
    "Digits only. What was the count of my list?",
    "Answer with digits only. How many things did I name?",
];

/// `(ordinal, adverb, index)`: `usize::MAX` is the last item.
const ORDINAL_SLOTS: &[(&str, &str, usize)] = &[
    ("first", "first", 0),
    ("second", "second", 1),
    ("third", "third", 2),
    ("last", "last", usize::MAX),
];

/// Turn-2 demands for the ordinal family; `{singular}` names the item kind.
const ORDINAL_DEMANDS: &[&str] = &[
    "One word only. What was the {ordinal} {singular}?",
    "Answer in one word. Which {singular} came {adverb}?",
    "One word only. Name the {ordinal} {singular}.",
    "Reply with one word. What was the {ordinal} item?",
    "Use one word only. Which {singular} was {adverb}?",
];

/// `(symbol, element)`: the flashcard pairs the name family reads in both directions.
const ELEMENT_FACTS: &[(&str, &str)] = &[
    ("Ar", "argon"),
    ("H", "hydrogen"),
    ("He", "helium"),
    ("C", "carbon"),
    ("N", "nitrogen"),
    ("O", "oxygen"),
    ("Na", "sodium"),
    ("Mg", "magnesium"),
    ("Al", "aluminium"),
    ("Si", "silicon"),
    ("Cl", "chlorine"),
    ("K", "potassium"),
    ("Ca", "calcium"),
    ("Fe", "iron"),
    ("Cu", "copper"),
    ("Zn", "zinc"),
    ("Ag", "silver"),
    ("Sn", "tin"),
    ("I", "iodine"),
    ("Au", "gold"),
    ("Hg", "mercury"),
    ("Pb", "lead"),
    ("Ne", "neon"),
    ("Ni", "nickel"),
];

/// `(country, capital)`: the second flashcard set, read in both directions.
const CAPITAL_FACTS: &[(&str, &str)] = &[
    ("France", "Paris"),
    ("Japan", "Tokyo"),
    ("Egypt", "Cairo"),
    ("Norway", "Oslo"),
    ("Italy", "Rome"),
    ("Spain", "Madrid"),
    ("Greece", "Athens"),
    ("Kenya", "Nairobi"),
    ("Peru", "Lima"),
    ("Cuba", "Havana"),
    ("Nepal", "Kathmandu"),
    ("Portugal", "Lisbon"),
];

const SYMBOL_PREMISES: &[&str] = &["Symbol: {symbol}.", "The symbol is {symbol}."];
const SYMBOL_DEMANDS: &[&str] = &[
    "Give a one-word answer. Which symbol did I give?",
    "One word only. What was the symbol I named?",
    "Answer in one word. What symbol did I give?",
    "Reply with one word. Which symbol was it?",
];
const ELEMENT_PREMISES: &[&str] = &["Element: {element}.", "The element is {element}."];
const ELEMENT_DEMANDS: &[&str] = &[
    "One word only. Which element did I name?",
    "Answer in one word. What element did I give?",
    "Give a one-word answer. Which element was it?",
    "Reply with one word. What was the element I named?",
];
const CAPITAL_PREMISES: &[&str] = &["Capital: {capital}.", "The capital is {capital}."];
const CAPITAL_DEMANDS: &[&str] = &[
    "One word only. Which capital did I name?",
    "Answer in one word. What capital did I give?",
    "Give a one-word answer. Which capital was it?",
    "Reply with one word. What was the capital I named?",
];
const COUNTRY_PREMISES: &[&str] = &["Country: {country}.", "The country is {country}."];
const COUNTRY_DEMANDS: &[&str] = &[
    "One word only. Which country did I name?",
    "Answer in one word. What country did I give?",
    "Give a one-word answer. Which country was it?",
    "Reply with one word. What was the country I named?",
];

/// `(object, kind, value)`: the one-clause facts the attribute family extracts a single word from.
const ATTRIBUTE_FACTS: &[(&str, &str, &str)] = &[
    ("rose", "colour", "red"),
    ("sky", "colour", "blue"),
    ("grass", "colour", "green"),
    ("snow", "colour", "white"),
    ("coal", "colour", "black"),
    ("lemon", "colour", "yellow"),
    ("ball", "shape", "round"),
    ("box", "shape", "square"),
    ("coin", "shape", "round"),
    ("brick", "material", "clay"),
    ("window", "material", "glass"),
    ("table", "material", "wood"),
    ("door", "material", "oak"),
    ("spoon", "material", "steel"),
    ("mouse", "size", "small"),
    ("whale", "size", "large"),
    ("ant", "size", "tiny"),
    ("brick", "weight", "heavy"),
    ("feather", "weight", "light"),
    ("stone", "weight", "heavy"),
    ("pillow", "feel", "soft"),
    ("ice", "feel", "cold"),
    ("sun", "feel", "warm"),
    ("knife", "edge", "sharp"),
];

/// One-clause premises: turn 1 states the attribute, its answer acknowledges in one short form, and
/// turn 2 asks for the attribute word. The value is therefore only ever in turn 1's USER turn.
const ATTRIBUTE_PREMISES: &[&str] = &[
    "The {object} is {value}.",
    "A {object} is {value}.",
    "The {object} I saw is {value}.",
];

/// Short acknowledgements: never the attribute and never the count, so copying turn 1's answer
/// always fails while the masked response stays 1-3 tokens long.
const ACKS: &[&str] = &["Noted.", "I see.", "Got it.", "Right.", "Understood."];

const ATTRIBUTE_DEMANDS: &[&str] = &[
    "One word only. What {kind} did I say the {object} was?",
    "Answer in one word. What was the {kind} of the {object}?",
    "Reply with one word. Which {kind} did I give the {object}?",
    "One word only. What {kind} was the {object}?",
    "Use one word only. What {kind} did I name for the {object}?",
];

/// `(word, value)`: turn 1's premise is the digits, its answer the short word form.
const NUMBER_FACTS: &[(&str, u64)] = &[
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
    ("thirteen", 13),
    ("fourteen", 14),
    ("fifteen", 15),
    ("sixteen", 16),
    ("seventeen", 17),
    ("eighteen", 18),
    ("nineteen", 19),
    ("twenty", 20),
    ("twenty-four", 24),
    ("thirty", 30),
    ("thirty-six", 36),
    ("forty", 40),
    ("fifty", 50),
    ("sixty", 60),
    ("seventy", 70),
    ("eighty", 80),
    ("ninety", 90),
    ("one hundred", 100),
];

const NUMBER_PREMISES: &[&str] = &[
    "The number is {value}.",
    "Number: {value}.",
    "My number is {value}.",
    "{value}.",
];

/// A numeric derivation over turn 1's number: the follow-up answer is computed, not copied.
#[derive(Clone, Copy)]
enum Op {
    Add(u64),
    Sub(u64),
    Mul(u64),
}

const NUMBER_DEMANDS: &[(&str, Op)] = &[
    (
        "Answer with digits only. What is that number minus one?",
        Op::Sub(1),
    ),
    ("Digits only. What is that number plus one?", Op::Add(1)),
    (
        "Answer with digits only. What is that number plus ten?",
        Op::Add(10),
    ),
    ("Digits only. What is that number minus ten?", Op::Sub(10)),
    (
        "Answer with digits only. What is that number times two?",
        Op::Mul(2),
    ),
    (
        "Reply with digits only. What is that number plus two?",
        Op::Add(2),
    ),
    (
        "Answer with digits only. What is that number minus two?",
        Op::Sub(2),
    ),
    ("Digits only. What is that number times ten?", Op::Mul(10)),
];

/// `(yes/no question, the short verdict turn 1 answers with)` — the polarity family's premises.
const JUDGEMENTS: &[(&str, &str)] = &[
    ("Is the sun a planet?", "No."),
    ("Is the sky blue?", "Yes."),
    ("Is ice hot?", "No."),
    ("Is grass green?", "Yes."),
    ("Are fish mammals?", "No."),
    ("Do birds fly?", "Yes."),
    ("Is water wet?", "Yes."),
    ("Is fire cold?", "No."),
    ("Is two even?", "Yes."),
    ("Is a whale a fish?", "No."),
    ("Is snow white?", "Yes."),
    ("Is coal black?", "Yes."),
    ("Do cats bark?", "No."),
    ("Is a rose a flower?", "Yes."),
    ("Is a stone soft?", "No."),
    ("Is the moon a star?", "No."),
    ("Does ice float?", "Yes."),
    ("Is the sun a star?", "Yes."),
    ("Are lemons sweet?", "No."),
    ("Is the sea salty?", "Yes."),
];

/// Every demand asks whether the previous answer was `no`, so the truthful answer is always the
/// COMPLEMENT of turn 1's answer: `Yes.` after `No.`, `No.` after `Yes.`.
const POLARITY_DEMANDS: &[&str] = &[
    "Reply with yes or no. Was my last answer no?",
    "Answer with yes or no. Was your previous answer no?",
    "Say yes or no. Did you just answer no?",
    "Answer yes or no only. Was your last answer no?",
    "Reply with a single word, yes or no. Was your previous answer no?",
];

fn count_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(category, _singular, items) in ITEM_LISTS {
        let list = comma_list(items);
        let count = items.len();
        let plural = category.to_lowercase();
        for ask in COUNT_ASKS {
            let question1 = fill(&fill(ask, "{cat}", category), "{list}", &list);
            for ack in ACKS {
                for demand in COUNT_DEMANDS {
                    specs.push(Spec {
                        family: "count",
                        question1: question1.clone(),
                        answer1: (*ack).to_string(),
                        question2: fill(demand, "{plural}", &plural),
                        answer2: format!("{count}."),
                    });
                }
            }
        }
    }
    specs
}

fn ordinal_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(category, singular, items) in ITEM_LISTS {
        let list = comma_list(items);
        for ask in COUNT_ASKS {
            let question1 = fill(&fill(ask, "{cat}", category), "{list}", &list);
            for &(ordinal, adverb, slot) in ORDINAL_SLOTS {
                let index = if slot == usize::MAX {
                    items.len() - 1
                } else {
                    slot
                };
                if index >= items.len() {
                    continue;
                }
                let item = items[index];
                for ack in ACKS {
                    for demand in ORDINAL_DEMANDS {
                        let question2 = fill(
                            &fill(&fill(demand, "{ordinal}", ordinal), "{adverb}", adverb),
                            "{singular}",
                            singular,
                        );
                        specs.push(Spec {
                            family: "ordinal",
                            question1: question1.clone(),
                            answer1: (*ack).to_string(),
                            question2: question2.clone(),
                            answer2: format!("{item}."),
                        });
                    }
                }
            }
        }
    }
    specs
}

fn name_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(symbol, element) in ELEMENT_FACTS {
        let element_title = capitalize(element);
        for premise in SYMBOL_PREMISES {
            let question1 = fill(premise, "{symbol}", symbol);
            for demand in SYMBOL_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    question1: question1.clone(),
                    answer1: format!("{element_title}."),
                    question2: (*demand).to_string(),
                    answer2: format!("{symbol}."),
                });
            }
        }
        for premise in ELEMENT_PREMISES {
            let question1 = fill(premise, "{element}", element);
            for demand in ELEMENT_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    question1: question1.clone(),
                    answer1: format!("{symbol}."),
                    question2: (*demand).to_string(),
                    answer2: format!("{element_title}."),
                });
            }
        }
    }
    for &(country, capital) in CAPITAL_FACTS {
        for premise in CAPITAL_PREMISES {
            let question1 = fill(premise, "{capital}", capital);
            for demand in CAPITAL_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    question1: question1.clone(),
                    answer1: format!("{country}."),
                    question2: (*demand).to_string(),
                    answer2: format!("{capital}."),
                });
            }
        }
        for premise in COUNTRY_PREMISES {
            let question1 = fill(premise, "{country}", country);
            for demand in COUNTRY_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    question1: question1.clone(),
                    answer1: format!("{capital}."),
                    question2: (*demand).to_string(),
                    answer2: format!("{country}."),
                });
            }
        }
    }
    specs
}

fn derive(value: u64, op: Op) -> Option<u64> {
    match op {
        Op::Add(addend) => value.checked_add(addend),
        Op::Sub(subtrahend) => value.checked_sub(subtrahend),
        Op::Mul(factor) => value.checked_mul(factor),
    }
}

fn number_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(word, value) in NUMBER_FACTS {
        for premise in NUMBER_PREMISES {
            let question1 = fill(premise, "{value}", &value.to_string());
            for (demand, op) in NUMBER_DEMANDS {
                let Some(derived) = derive(value, *op) else {
                    continue;
                };
                if derived == value {
                    continue;
                }
                specs.push(Spec {
                    family: "number",
                    question1: question1.clone(),
                    answer1: format!("{}.", capitalize(word)),
                    question2: (*demand).to_string(),
                    answer2: format!("{derived}."),
                });
            }
        }
    }
    specs
}

fn polarity_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(question1, answer1) in JUDGEMENTS {
        let answer2 = if answer1 == "No." { "Yes." } else { "No." };
        for demand in POLARITY_DEMANDS {
            specs.push(Spec {
                family: "polarity",
                question1: question1.to_string(),
                answer1: answer1.to_string(),
                question2: (*demand).to_string(),
                answer2: answer2.to_string(),
            });
        }
    }
    specs
}

fn attribute_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(object, kind, value) in ATTRIBUTE_FACTS {
        for premise in ATTRIBUTE_PREMISES {
            let question1 = fill(&fill(premise, "{object}", object), "{value}", value);
            for ack in ACKS {
                for demand in ATTRIBUTE_DEMANDS {
                    specs.push(Spec {
                        family: "attribute",
                        question1: question1.clone(),
                        answer1: (*ack).to_string(),
                        question2: fill(&fill(demand, "{kind}", kind), "{object}", object),
                        answer2: format!("{value}."),
                    });
                }
            }
        }
    }
    specs
}

/// The attribute word as the one-word answer form demands it (sentence-initial capital).
fn capitalize(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
    }
}

fn candidate_families() -> Vec<(&'static str, Vec<Spec>)> {
    vec![
        ("count", count_specs()),
        ("ordinal", ordinal_specs()),
        ("name", name_specs()),
        ("number", number_specs()),
        ("polarity", polarity_specs()),
        ("attribute", attribute_specs()),
    ]
}

/// The response-content token length of an authored answer, measured with the bound tokenizer
/// exactly as `build_documents` will emit it (leading space included).
fn answer_tokens(tokenizer: &HfBpeTokenizer, answer: &str) -> usize {
    encode_with(tokenizer, &format!(" {answer}")).len()
}

/// Filter every family to the length gate, prefer the shortest answers, truncate to the quota and
/// then shuffle the union: the document order and membership are fixed by the seed alone, and no
/// candidate can be emitted twice. The selection is deterministic for a given seed and tokenizer.
fn select_specs(
    seed: u64,
    tokenizer: &HfBpeTokenizer,
) -> Result<(Vec<Spec>, BTreeMap<&'static str, (usize, usize)>), String> {
    let mut selected: Vec<Spec> = Vec::new();
    let mut report: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for (family, specs) in candidate_families() {
        let enumerated = specs.len();
        if enumerated == 0 {
            return Err(format!("family {family} enumerated no candidates"));
        }
        let quota = quota_for(family)?;
        // The length gate: reject any candidate whose emitted answers are too long.
        let mut fitting: Vec<(Spec, usize)> = Vec::new();
        let mut rejected = 0usize;
        let mut histogram: BTreeMap<usize, usize> = BTreeMap::new();
        for spec in specs {
            let length1 = answer_tokens(tokenizer, &spec.answer1);
            let length2 = answer_tokens(tokenizer, &spec.answer2);
            if length1 == 0 || length2 == 0 {
                return Err(format!(
                    "family {family} produced an empty answer (turn 1 {:?}, turn 2 {:?})",
                    spec.answer1, spec.answer2
                ));
            }
            *histogram.entry(length1).or_default() += 1;
            if length1 > MAX_ANSWER1_TOKENS || length2 > MAX_ANSWER2_TOKENS {
                rejected += 1;
                continue;
            }
            fitting.push((spec, length1));
        }
        if fitting.len() < quota {
            return Err(format!(
                "family {family} has only {} candidates inside the length gate ({rejected} rejected \
                 over {MAX_ANSWER1_TOKENS}/{MAX_ANSWER2_TOKENS} tokens) and needs {quota}; turn-1 \
                 answer length histogram (tokens: candidates) {histogram:?}",
                fitting.len()
            ));
        }
        shuffle(&mut fitting, family_seed(seed, family));
        // A stable sort by length class keeps the seeded order inside each class and selects the
        // shortest candidates first, so the emitted mean lands inside the target rather than at the
        // ceiling.
        fitting.sort_by_key(|(_, length)| usize::from(*length > PREFERRED_ANSWER1_TOKENS));
        fitting.truncate(quota);
        report.insert(family, (enumerated, fitting.len()));
        selected.extend(fitting.into_iter().map(|(spec, _)| spec));
    }
    let total: usize = report.values().map(|entry| entry.1).sum();
    if !(MIN_DOCUMENTS..=MAX_DOCUMENTS).contains(&total) {
        return Err(format!(
            "only {total} documents were selected; the store must hold {MIN_DOCUMENTS} to \
             {MAX_DOCUMENTS}"
        ));
    }
    shuffle(&mut selected, seed ^ 0xA5A5_5A5A_1234_5678);
    Ok((selected, report))
}

fn family_seed(seed: u64, family: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in family.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    seed ^ hash
}

fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed;
    for index in (1..items.len()).rev() {
        let pick = (splitmix64(&mut state) % (index as u64 + 1)) as usize;
        items.swap(index, pick);
    }
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

/// Build one two-turn document in the chat corpus's framing, recording both answers' response
/// content so the transformation census reads exactly what was emitted.
fn build_documents(specs: &[Spec], tokenizer: &HfBpeTokenizer) -> Result<Vec<Doc>, String> {
    let mut documents = Vec::with_capacity(specs.len());
    for spec in specs {
        let mut tokens: Vec<u16> = Vec::new();
        let mut mask: Vec<u8> = Vec::new();
        push(&mut tokens, &mut mask, &[BOS_ID as u16], 0);
        push(&mut tokens, &mut mask, &USER_MARKER.map(|id| id as u16), 0);
        push(
            &mut tokens,
            &mut mask,
            &encode_with(tokenizer, &format!(" {}", spec.question1)),
            0,
        );
        push(&mut tokens, &mut mask, &[SEPARATOR_ID], 0);
        push(
            &mut tokens,
            &mut mask,
            &ASSISTANT_MARKER.map(|id| id as u16),
            0,
        );
        let answer1_ids = encode_with(tokenizer, &format!(" {}", spec.answer1));
        push(&mut tokens, &mut mask, &answer1_ids, 1);
        push(&mut tokens, &mut mask, &[EOS_ID as u16], 1);
        // Every turn after the first is preceded by the single `\n` separator.
        push(&mut tokens, &mut mask, &[SEPARATOR_ID], 0);
        push(&mut tokens, &mut mask, &USER_MARKER.map(|id| id as u16), 0);
        push(
            &mut tokens,
            &mut mask,
            &encode_with(tokenizer, &format!(" {}", spec.question2)),
            0,
        );
        push(&mut tokens, &mut mask, &[SEPARATOR_ID], 0);
        push(
            &mut tokens,
            &mut mask,
            &ASSISTANT_MARKER.map(|id| id as u16),
            0,
        );
        let response2_start = tokens.len();
        let answer2_ids = encode_with(tokenizer, &format!(" {}", spec.answer2));
        push(&mut tokens, &mut mask, &answer2_ids, 1);
        push(&mut tokens, &mut mask, &[EOS_ID as u16], 1);
        if answer1_ids.is_empty() || answer2_ids.is_empty() {
            return Err(format!(
                "family {} produced an empty answer (turn 1 {:?}, turn 2 {:?})",
                spec.family, spec.answer1, spec.answer2
            ));
        }
        documents.push(Doc {
            tokens,
            mask,
            family: spec.family,
            answer1_ids,
            answer2_ids,
            response2_start,
            answer1_text: spec.answer1.clone(),
            answer2_text: spec.answer2.clone(),
        });
    }
    Ok(documents)
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

/// Is `needle` a contiguous run of `haystack`?
fn contains_span(haystack: &[u16], needle: &[u16]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// (b) How many documents a verbatim copy of turn 1's answer would pass, per family, plus the
/// weaker span-copy readings.
fn relation_census(documents: &[Doc]) -> RelationStats {
    let mut stats = RelationStats {
        documents: documents.len(),
        equal: 0,
        span_in_answer1: 0,
        span_in_prefix: 0,
        per_family: BTreeMap::new(),
    };
    for document in documents {
        let equal = document.answer2_ids == document.answer1_ids;
        let span_in_answer1 = contains_span(&document.answer1_ids, &document.answer2_ids);
        let span_in_prefix = contains_span(
            &document.tokens[..document.response2_start],
            &document.answer2_ids,
        );
        stats.equal += usize::from(equal);
        stats.span_in_answer1 += usize::from(span_in_answer1);
        stats.span_in_prefix += usize::from(span_in_prefix);
        let entry = stats.per_family.entry(document.family).or_default();
        entry.documents += 1;
        entry.equal += usize::from(equal);
        entry.span_in_answer1 += usize::from(span_in_answer1);
        entry.span_in_prefix += usize::from(span_in_prefix);
    }
    stats
}

fn report_relation(stats: &RelationStats) {
    let documents = stats.documents;
    let not_equal = documents - stats.equal;
    println!(
        "transformation census (b): {not_equal}/{documents} documents ({:.6}) have a turn-2 answer \
         that is NOT token-identical to turn 1's answer; identical (a copy policy would pass): {}",
        not_equal as f64 / documents as f64,
        stats.equal
    );
    println!(
        "  weaker readings (annex): turn-2 answer is a contiguous span of turn-1's answer {} \
         ({:.6}), of the whole prefix before turn 2 {} ({:.6}); NOT a contiguous span of the prefix \
         {} ({:.6})",
        stats.span_in_answer1,
        stats.span_in_answer1 as f64 / documents as f64,
        stats.span_in_prefix,
        stats.span_in_prefix as f64 / documents as f64,
        documents - stats.span_in_prefix,
        (documents - stats.span_in_prefix) as f64 / documents as f64
    );
    for (family, entry) in &stats.per_family {
        println!(
            "  family {family}: {} documents, identical {}, span-of-turn-1 {}, span-of-prefix {}",
            entry.documents, entry.equal, entry.span_in_answer1, entry.span_in_prefix
        );
    }
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
            if mask[marker_start..run_start]
                .iter()
                .any(|&value| value != 0)
            {
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
fn validate_turn_units(documents: &[Doc]) -> Ledger {
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

/// (c) Prove the validator bites: corrupt one document in memory only, show the offending document
/// index and clause, and re-run the clean validation to show the store itself is untouched.
fn corruption_test(
    title: &str,
    stream: &[u16],
    mask: &[u8],
    document: usize,
) -> Result<(), String> {
    let starts = document_starts(stream);
    if document + 1 >= starts.len() {
        return Err(format!(
            "corruption test document {document} is out of range"
        ));
    }
    let (start, end) = (starts[document], starts[document + 1]);
    println!(
        "corruption test ({title}): document {document} spans [{start},{end}); leaving its terminal \
         EOS unmasked in memory only"
    );

    let mut corrupted = mask.to_vec();
    corrupted[end - 1] = 0;
    let ledger = validate_documents(stream, &corrupted);
    println!(
        "  case (terminal EOS of document {document} unmasked): {} offenders over {} clauses",
        ledger.offenders(),
        ledger.counts.len()
    );
    for (clause, count) in &ledger.counts {
        if *count > 0 {
            println!("    clause {clause}: {count}");
        }
    }
    for (offending, clause, detail) in ledger.offenders.iter().take(4) {
        println!("    offending document {offending} clause {clause}: {detail}");
    }
    if ledger.offenders() == 0 {
        return Err(format!(
            "corruption test ({title}) was NOT detected; the validator does not bite"
        ));
    }
    if !ledger
        .offenders
        .iter()
        .any(|(offending, _, _)| *offending == document)
    {
        return Err(format!(
            "corruption test ({title}) fired but not for document {document}"
        ));
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
    match EpisodeIndex::new(stream, &corrupted, contract, &sources) {
        Ok(index) => println!(
            "    training-path EpisodeIndex admitted the corrupted store: documents {}, runs {}, \
             excluded {}",
            index.population().documents,
            index.population().response_runs,
            index.population().excluded_over_context
        ),
        Err(error) => println!("    training-path EpisodeIndex rejected it: {error}"),
    }

    // Restore: the same validator over the untouched bytes.
    let restored = validate_documents(stream, mask);
    println!(
        "  restored (original bytes): {} offenders over {} clauses",
        restored.offenders(),
        restored.counts.len()
    );
    if restored.offenders() != 0 {
        return Err(format!(
            "the restored store did not validate clean ({title})"
        ));
    }
    Ok(())
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

/// Write the token store and the response mask.
fn write_store_files(directory: &Path, stream: &[u16], mask: &[u8]) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|e| format!("create {}: {e}", directory.display()))?;
    let tokens_path = directory.join("tokens.u16");
    CorpusWriter::write_file(&tokens_path, VOCAB_SIZE, stream)
        .map_err(|e| format!("write {}: {e}", tokens_path.display()))?;
    let mask_path = directory.join("response_mask.u8");
    fs::write(&mask_path, mask).map_err(|e| format!("write {}: {e}", mask_path.display()))?;
    Ok(())
}

/// Write the manifest. `reader` is the population the training path's own `EpisodeIndex` reported
/// for these exact bytes; it is recorded only after that load succeeded, and the caller re-loads
/// the store after this write so the reported manifest is the one that was accepted.
#[allow(clippy::too_many_arguments)]
fn write_manifest(
    directory: &Path,
    stream: &[u16],
    mask: &[u8],
    args: &Args,
    tokenizer_sha256: &str,
    family_report: &BTreeMap<&'static str, (usize, usize)>,
    relation: &RelationStats,
    lengths: &LengthStats,
    transformation_fraction: f64,
    reader: Option<&uor_r4_training::dialogue_episodes::EpisodePopulation>,
) -> Result<(), String> {
    let tokens_path = directory.join("tokens.u16");
    let families: BTreeMap<&str, serde_json::Value> = family_report
        .iter()
        .map(|(family, (enumerated, emitted))| {
            (
                *family,
                serde_json::json!({"candidates": enumerated, "documents": emitted}),
            )
        })
        .collect();
    let per_family: BTreeMap<&str, serde_json::Value> = relation
        .per_family
        .iter()
        .map(|(family, entry)| {
            (
                *family,
                serde_json::json!({
                    "documents": entry.documents,
                    "answer2_equals_answer1": entry.equal,
                    "answer2_span_of_answer1": entry.span_in_answer1,
                    "answer2_span_of_prefix": entry.span_in_prefix,
                }),
            )
        })
        .collect();

    let mut manifest = serde_json::json!({
        "dialogue_protocol": "uor-r4.literal-role-dialogue/2",
        "documents": family_report.values().map(|entry| entry.1).sum::<usize>(),
        "drops": {"special_token_occurrences": 0},
        "families": families,
        "files": [{
            "label": OUTPUT_LABEL,
            "tokens": stream.len(),
            "special_token_occurrences": 0,
        }],
        "generator": "crates/uor-r4-training/examples/demand-store-attrshort.rs",
        "mask_bytes": mask.len(),
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating \
                      <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, \
                      and an unmasked document-terminal <|eos|>.",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "mask_sha256": uor_r4_training::sha256_bytes(mask),
        "response_tokens": mask.iter().filter(|&&value| value == 1).count(),
        "schema": "uor-r4-demand-store-attrshort/v1",
        "seed": format!("0x{:016x}", args.seed),
        // Key parity with `demand-store-2turn/manifest.json`: that tool derives its documents from
        // a source store, this one synthesises them, so the key is present and explicitly null.
        "source_store": serde_json::Value::Null,
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
        "transformation": {
            "documents": relation.documents,
            "answer2_equals_answer1": relation.equal,
            "not_equal_fraction": transformation_fraction,
            "answer2_span_of_answer1": relation.span_in_answer1,
            "answer2_span_of_prefix": relation.span_in_prefix,
            "min_required_fraction": MIN_TRANSFORMATION_FRACTION,
            "per_family": per_family,
        },
        // (b) What the store teaches as answers, in response-content tokens (EOS excluded).
        "answer_lengths": {
            "turn1": {
                "mean": lengths.mean1,
                "max": lengths.max1,
                "total": lengths.total1,
                "histogram": lengths.histogram1,
            },
            "turn2": {
                "mean": lengths.mean2,
                "max": lengths.max2,
                "total": lengths.total2,
                "histogram": lengths.histogram2,
            },
            "response_content_tokens": lengths.total1 + lengths.total2,
            "response_tokens_including_eos": mask.iter().filter(|&&value| value == 1).count(),
            "target_mean_turn1_max": TARGET_ANSWER1_MEAN,
            "max_turn1_tokens": MAX_ANSWER1_TOKENS,
            "max_turn2_tokens": MAX_ANSWER2_TOKENS,
            "compliant_store_response_tokens": COMPLIANT_STORE_RESPONSE_TOKENS,
            "mass_fraction_of_compliant_store": (mask.iter().filter(|&&value| value == 1).count()
                as f64)
                / (COMPLIANT_STORE_RESPONSE_TOKENS as f64),
        },
    });
    if let Some(population) = reader {
        if let Some(object) = manifest.as_object_mut() {
            object.insert(
                "reader".to_string(),
                serde_json::json!({
                    "context": EPISODE_CONTEXT,
                    "documents": population.documents,
                    "response_runs": population.response_runs,
                    "response_tokens": population.response_tokens,
                    "admitted_responses": population.eligible_responses,
                    "admitted_response_tokens": population.eligible_response_tokens,
                    "excluded_over_context": population.excluded_over_context,
                    "identical_prefixes": population.identical_prefixes,
                    "source": "DialogueSplit::load + EpisodeIndex",
                }),
            );
        } else {
            return Err("manifest is not a JSON object".to_string());
        }
    }
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
        || manifest["tokens_sha256"].as_str()
            != Some(
                uor_r4_training::sha256_file(&tokens_path)
                    .map_err(|e| e.to_string())?
                    .as_str(),
            )
        || manifest["tokens"].as_u64() != Some(stream.len() as u64)
        || manifest["tokens_bytes"].as_u64() != Some((CORPUS_HEADER_SIZE + stream.len() * 2) as u64)
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
    Ok(population)
}
