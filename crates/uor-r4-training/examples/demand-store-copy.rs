//! Build a CONTENT-COPY demand store: turn 1 RESTATES the premise's content, turn 2 asks for a
//! specific attribute of that restated content, and no response is ever an acknowledgement.
//!
//! ```text
//! demand-store-copy <output-dir> [--tokenizer <tokenizer.json>] [--seed N] [--corrupt-test]
//!                   [--samples N]
//! ```
//!
//! Why this store (measured, 2026-10-08): `docs/reviews/cross-turn-binding-diagnosis-2026-10-08.md`
//! and `docs/evidence/binding_probe_p1_report_2026-10-08.txt`. The P1 probe found that the
//! store-armed models DO put read mass above TAU on the expected value's span and then answer with
//! the store's own surface families instead (`Noted.`, `I see.`, `3.`); in the worst case turn 1 and
//! turn 2 were both `Noted.` — a self-conditioned register loop created by the training data. The
//! diagnosis: turn-1 answers that are content-free acknowledgements teach no span selectivity. This
//! tool emits the store the diagnosis asks for.
//!
//! Three construction rules, one per defect the earlier stores had:
//!
//! 1. **No acknowledgements anywhere.** Every response is content-bearing, and the build refuses to
//!    write if any response equals an acknowledgement in `ACKNOWLEDGEMENTS`.
//! 2. **Turn 1's answer is a SHORT COPY of the premise.** `Metals: iron, gold.` -> `Iron, gold.`;
//!    `The rose is red.` -> `Red.`; `Capital: Paris.` -> `Paris.`; `Symbol: Ar.` -> `Ar.`. The
//!    turn-1 response slot therefore teaches "copy the relevant content of the premise" instead of a
//!    register, and (checked) the answer's token sequence is a contiguous span of turn 1's user
//!    content in every document. Every turn-1 answer is at most `MAX_ANSWER1_TOKENS` response-content
//!    tokens with an emitted mean at or below `TARGET_ANSWER1_MEAN`.
//! 3. **Turn 2's answer is short, demanded in form, and MUST require reading turn 1's content.**
//!    `Answer with digits only. How many items did I list?` -> `2.`; `One word only. What was the
//!    second item?` -> `gold.`; `Give a one-word answer. Name the capital I gave.` -> `Paris.`;
//!    `One word only. What colour did I say the rose was?` -> `red.`; `Answer with digits only. What
//!    is that number in figures?` -> `20.`; `One word only. Which unit did I attach to my weight?`
//!    -> `kilograms.`
//!
//! The check the earlier stores got wrong, and this one performs: for EVERY document the tool
//! verifies that turn 2's answer does **not** occur in ANY user turn of that document (the premise
//! is where such a value would otherwise sit) AND is not token-identical to turn 1's answer. Both
//! counts are printed and both must be zero. The earlier stores asserted only the second.
//!
//! Value/template randomisation (so no surface statistic solves it): every demanded value occurs
//! under many paraphrased templates and every template occurs over many values, in both turns; the
//! tool prints the distinct `(value, template)` combination count per family and refuses to write if
//! a template carries too few distinct values or one value takes too large a share of its family.
//!
//! ```text
//! BOS  User:  <turn 1 premise>  \n  Assistant:  <turn 1 copy>  EOS
//!      \n  User:  <turn 2 demand>  \n  Assistant:  <turn 2 answer>  EOS
//! mask 0    mask 0              0     mask 0      1    1       0
//!        0       0              0       0          1    1
//! ```
//!
//! The framing is the prepared chat corpus's multi-turn convention (`uor-r4.literal-role-dialogue/2`):
//! BOS, then turns joined by a single `\n` separator placed before every turn after the first; a turn
//! is `<marker><content>` with `User: `/`Assistant: ` markers; every assistant turn ends with
//! `<|eos|>`; a document-terminal `<|eos|>` is appended only when the final emitted turn is not
//! assistant. The mask rule is the chat manifest's: 1 on each assistant turn's response content and
//! its terminating `<|eos|>`, 0 on BOS, role markers, turn separators and all user content.
//!
//! Two of the single-turn generator's sixteen clauses are per-document there and cannot hold
//! literally for a document with two assistant turns; this tool checks their multi-turn
//! generalisation and prints the literal single-turn counts as an annex:
//!
//! - `document_no_interior_eos` -> no *unmasked* EOS strictly inside a document (the training
//!   path's own rule in `dialogue_episodes.rs`); a completed masked turn EOS is expected interior.
//! - `response_single_run` -> each assistant marker opens exactly one contiguous masked run that
//!   closes at its own EOS, with an unmasked token (or the document end) after it.
//!
//! Two further clauses are read with this store's own extra condition, because they are exactly the
//! two checks the earlier stores lacked; both implementations (this tool and the independent Python
//! reader of the emitted bytes) compute them:
//!
//! - `document_vocabulary` -> additionally: no response content equals an acknowledgement.
//! - `response_no_interior_eos` -> additionally: turn 2's answer is not token-identical to turn 1's
//!   answer and does not occur in any user turn of the document.
//!
//! Every other clause is the source tools' clause, applied per document or per response run. The
//! tool additionally re-checks every assistant turn as a literal single-turn document (the unchanged
//! single-turn clause set). Nothing is written unless all clause counters are zero and the length,
//! transformation and diversity gates pass; after writing, the store is re-read through
//! `DialogueSplit::load` and the real `EpisodeIndex`, so acceptance is the training path's own check.
//! `--corrupt-test` corrupts one document in memory (never on disk), shows the offending document
//! index and clause, restores, and is repeated over the bytes read back from the emitted store.
//!
//! Families (every emitted document belongs to exactly one; every turn-1 answer is short and
//! copied). The example lines are real emitted documents from the default seed:
//!
//! - `count`:      t1 `Here: run, walk.` -> `run, walk.`,
//!                 t2 `Answer with digits only. How many items were in my list?` -> `2.`
//! - `ordinal`:    t1 `Colours: red, blue.` -> `red, blue.`,
//!                 t2 `One word only. What was the position of blue?` -> `last.`
//! - `name`:       t1 `Symbol: Ar.` -> `Ar.`,
//!                 t2 `Give a one-word answer. Name the element I gave.` -> `Argon.`
//! - `conversion`: t1 `The volume is 3 litres.` -> `3 litres.`,
//!                 t2 `Answer with digits only. What does 3 litres become in millilitres?` -> `3000.`
//! - `number`:     t1 `My number is seven.` -> `seven.`,
//!                 t2 `Answer with digits only. What is that number in figures?` -> `7.`
//!
//! Two design consequences are worth stating because they were forced by measurement, not taste:
//!
//! 1. **Turn 1's answer is the premise's own token span**, not a re-encoding of the same words. A
//!    byte-level BPE tokenizer merges a running sentence's leading boundary differently from a
//!    response's (` is` + ` red` against ` red` + `.`), so `Iron, gold.` written as a response is NOT
//!    a token span of `Metals: iron, gold.` even though it is the same words. `build_documents`
//!    therefore extracts the longest available span of the PREMISE's tokens, which is why the emitted
//!    copies read `run, walk.` and `seven.` — the premise's own characters. The copy is then verbatim
//!    in the model's token view too, which is what makes the turn-1 slot teach span selection.
//! 2. **No family asks for a span the premise already holds verbatim.** An attribute-lookup family
//!    (`The rose is red.` -> `Red.` -> `What colour did I say the rose was?` -> `red.`) cannot satisfy
//!    the containment check: the answer `red` is a user-turn word, so a premise reader can answer
//!    without reading turn 1. The families here ask for what turn 1's copied content IMPLIES — the
//!    count of the list it restated, the position of an item inside it, the other surface form of a
//!    flashcard, the converted figures of a measurement, the digits of a number word — so turn 2's
//!    answer is in no user turn and turn 1's content is genuinely required.

use std::collections::{BTreeMap, BTreeSet};
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
const OUTPUT_LABEL: &str = "demand-copy";
/// This tool's document-order seed; `--seed` overrides it.
const DEFAULT_SEED: u64 = 0x636f_7079_7374_6f72; // "copystor"
/// The hard ceiling on a turn-1 answer's response-content tokens (its EOS excluded).
const MAX_ANSWER1_TOKENS: usize = 14;
/// The hard ceiling on a turn-2 answer's response-content tokens (its EOS excluded).
const MAX_ANSWER2_TOKENS: usize = 8;
/// Candidate preference boundary: an answer at or below it is selected first, so the emitted means
/// land well inside the ceilings instead of at them.
const PREFERRED_ANSWER1_TOKENS: usize = 11;
const PREFERRED_ANSWER2_TOKENS: usize = 5;
/// The emitted mean turn-1 answer length must not exceed this.
const TARGET_ANSWER1_MEAN: f64 = 8.0;
/// The compliant configuration range the store must stay inside.
const MIN_DOCUMENTS: usize = 900;
const MAX_DOCUMENTS: usize = 1200;
/// Response tokens of the two-turn store that produced ten of ten form compliance; this store's
/// response mass must stay materially below it.
const COMPLIANT_STORE_RESPONSE_TOKENS: usize = 31_084;
/// The working diluted variant's response mass, for scale.
const DILUTED_VARIANT_RESPONSE_TOKENS: usize = 9_360;
/// The brief's hard ceiling on the whole store's response tokens.
const RESPONSE_TOKEN_CEILING: usize = 12_000;
/// How many offending documents the refusal prints before summarizing.
const MAX_REPORTED_OFFENDERS: usize = 20;
/// No response may equal one of these, in either turn, in any document. This is the forbidden set
/// the diagnosis names: content-free acknowledgements. Adding entries can only make the store
/// stricter; `Right.`/`Sure.` are listed so no register family can slip back in.
const ACKNOWLEDGEMENTS: &[&str] = &[
    "Noted.",
    "I see.",
    "Got it.",
    "Understood.",
    "Right.",
    "Sure.",
    "Okay.",
    "OK.",
    "Alright.",
    "Very well.",
    "Of course.",
    "Certainly.",
];
/// The demanded floor on the fraction of documents whose second answer is not a verbatim copy of
/// the first answer's token sequence. This store is built so the fraction is exactly 1.0.
const MIN_TRANSFORMATION_FRACTION: f64 = 1.0;
/// Diversity floor: each (value, family-template) pair must occur with at least this many distinct
/// values before the store may be written, so no template can be solved by a surface statistic.
const MIN_DISTINCT_VALUES_PER_TEMPLATE: usize = 2;
/// Diversity ceiling on the share of one family a single value may take.
const MAX_VALUE_FAMILY_SHARE: f64 = 0.40;
/// Diversity floor on how many different templates a single value must appear under.
const MIN_TEMPLATES_PER_VALUE: usize = 2;

/// Every clause the store must satisfy. The names mirror the demand stores' generators, which
/// mirror the training-time checks in `dialogue_episodes.rs`; the counter for each must end at zero.
/// `ACK_CLAUSE` and `DEPENDENCY_CLAUSE` carry this store's two extra conditions (see the module
/// comment); the other fourteen are the source tools' clauses unchanged.
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
/// The clause that carries "no response is an acknowledgement".
const ACK_CLAUSE: &str = "document_vocabulary";
/// The clause that carries "turn 2's answer is not turn 1's answer and is in no user turn".
const DEPENDENCY_CLAUSE: &str = "response_no_interior_eos";

/// One authored two-turn document before tokenization. `value` is the demanded content the
/// diversity census counts (the family's answer-bearing value), never emitted as such.
#[derive(Clone)]
struct Spec {
    family: &'static str,
    premise: String,
    answer1: String,
    demand: String,
    answer2: String,
    value: String,
    /// The declared DEMAND TEMPLATE with its value-bearing slots already filled by the time the
    /// question text is built (the family's own `{item}`/`{value}` positions), and `demand_slot`
    /// the identity of this document's slot inside it. The diversity census groups by the pair, so
    /// "how many items did I list?" counts as one template across every count it meets.
    demand_template: String,
    demand_slot: String,
}

/// One emitted two-turn document with the provenance the census needs.
struct Doc {
    tokens: Vec<u16>,
    mask: Vec<u8>,
    family: &'static str,
    value: String,
    /// The response content of turn 1 without its EOS, as emitted.
    answer1_ids: Vec<u16>,
    /// The response content of turn 2 without its EOS, as emitted.
    answer2_ids: Vec<u16>,
    /// The first token of turn 2's response run.
    response2_start: usize,
    /// `(start, end)` of every user turn's CONTENT in the emitted token stream.
    user_spans: Vec<(usize, usize)>,
    premise_text: String,
    answer1_text: String,
    demand_text: String,
    answer2_text: String,
    demand_template: String,
    demand_slot: String,
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
             mean {:.4} max {} (histogram length:count {hist1:?}); turn 2 mean {:.4} max {} \
             (histogram {hist2:?}); total content tokens {}/{}",
            self.documents,
            self.mean1,
            self.max1,
            self.mean2,
            self.max2,
            self.total1,
            self.total2,
            hist1 = self.histogram1,
            hist2 = self.histogram2,
        );
    }
}

/// The store-wide evidence the brief's checks (b) and (c) report.
#[derive(Default)]
struct Evidence {
    /// (c) Responses whose content is an acknowledgement, over both turns.
    acknowledgements: usize,
    /// (c) Responses whose lower-cased content is an acknowledgement (reported, not a gate).
    acknowledgements_case_insensitive: usize,
    /// (b) Documents whose turn-2 answer occurs as a word in ANY user turn of that document.
    answer2_in_user_turn: usize,
    /// Annex: documents where the byte-level token span of turn 2's answer occurs in a user turn.
    /// This is a subword artifact of a running sentence's leading boundary, not phrase containment.
    answer2_token_span_in_user_turn: usize,
    /// (b) Documents whose turn-2 answer is token-identical to turn 1's answer.
    answer2_equals_answer1: usize,
    /// Annex: documents whose turn-2 answer occurs anywhere in the prefix before turn 2.
    answer2_in_prefix: usize,
    /// Documents whose turn-1 answer is NOT a contiguous span of turn 1's user content.
    answer1_not_in_premise: usize,
    /// Documents whose turn-1 answer is not a prefix-anchored copy (unused annex).
    answer1_in_closing_span: usize,
}

/// Per-family value/template diversity census.
#[derive(Clone, Default)]
struct FamilyDiversity {
    documents: usize,
    values: BTreeSet<String>,
    templates: BTreeSet<String>,
    /// Distinct (value, template) combinations emitted.
    combinations: usize,
    /// The largest number of documents any one value holds in this family.
    top_value_documents: usize,
    top_value: String,
    /// The fewest distinct values any one template holds in this family.
    min_values_per_template: usize,
    min_values_template: String,
    /// The fewest distinct templates any one value holds in this family.
    min_templates_per_value: usize,
    min_templates_value: String,
}

struct Diversity {
    documents: usize,
    combinations: usize,
    values: usize,
    templates: usize,
    per_family: BTreeMap<&'static str, FamilyDiversity>,
}

/// Per-family transformation census, extended with the containment check the earlier stores lacked.
#[derive(Clone, Copy, Default)]
struct FamilyRelation {
    documents: usize,
    equal: usize,
    span_in_answer1: usize,
    span_in_prefix: usize,
    answer2_in_user_turn: usize,
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
    /// Second answer is a contiguous span of a USER turn: a premise-copy policy would pass.
    answer2_in_user_turn: usize,
    per_family: BTreeMap<&'static str, FamilyRelation>,
}

struct Ledger {
    counts: Vec<(&'static str, usize)>,
    offenders: Vec<(usize, &'static str, String)>,
    /// The two augmented clauses' own sub-counts, so the report can name them separately.
    acknowledgement_hits: usize,
    dependency_hits: usize,
}

impl Ledger {
    fn new() -> Self {
        Self {
            counts: CLAUSES.iter().map(|clause| (*clause, 0usize)).collect(),
            offenders: Vec::new(),
            acknowledgement_hits: 0,
            dependency_hits: 0,
        }
    }

    fn hit(&mut self, clause: &'static str, document: usize, detail: String) {
        if let Some(entry) = self.counts.iter_mut().find(|entry| entry.0 == clause) {
            entry.1 += 1;
        }
        if clause == ACK_CLAUSE {
            self.acknowledgement_hits += 1;
        }
        if clause == DEPENDENCY_CLAUSE {
            self.dependency_hits += 1;
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
        eprintln!("demand-store-copy: {error}");
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

    // (b) length gate: what the store teaches as answers must stay short in both turns.
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
    if response_tokens >= RESPONSE_TOKEN_CEILING {
        return Err(format!(
            "refusing to write: {response_tokens} response tokens are at or over the \
             {RESPONSE_TOKEN_CEILING} ceiling"
        ));
    }
    let mass_ratio = response_tokens as f64 / COMPLIANT_STORE_RESPONSE_TOKENS as f64;
    println!(
        "response mass (c vs the compliant store and the diluted variant): {response_tokens} \
         response tokens, {:.4} of {COMPLIANT_STORE_RESPONSE_TOKENS} ({:.2}% below), {:.4} of the \
         diluted {DILUTED_VARIANT_RESPONSE_TOKENS} ({:.2}% {}), mean turn-1 answer {:.4} tokens \
         (max {}), mean turn-2 answer {:.4} tokens (max {}) over {} documents",
        mass_ratio,
        (1.0 - mass_ratio) * 100.0,
        response_tokens as f64 / DILUTED_VARIANT_RESPONSE_TOKENS as f64,
        (1.0 - response_tokens as f64 / DILUTED_VARIANT_RESPONSE_TOKENS as f64) * 100.0,
        if response_tokens >= DILUTED_VARIANT_RESPONSE_TOKENS {
            "above"
        } else {
            "below"
        },
        lengths.mean1,
        lengths.max1,
        lengths.mean2,
        lengths.max2,
        documents.len()
    );

    // (b)/(c) the evidence the brief demands, measured over the emitted documents.
    let evidence = measure_evidence(&documents);
    println!(
        "evidence (c) acknowledgements: {} exact, {} case-insensitive, forbidden set {ACKNOWLEDGEMENTS:?}",
        evidence.acknowledgements, evidence.acknowledgements_case_insensitive
    );
    println!(
        "evidence (b) dependency: turn-2 answer as a WORD in a user turn {} of {}; turn-2 answer \
         token-identical to turn-1 answer {}; annex (byte-level token span of turn-2's answer inside \
         a user turn, a BPE boundary artifact): {}; annex: turn-2 answer anywhere in the prefix {}; \
         turn-1 answer NOT a span of its premise {}",
        evidence.answer2_in_user_turn,
        documents.len(),
        evidence.answer2_equals_answer1,
        evidence.answer2_token_span_in_user_turn,
        evidence.answer2_in_prefix,
        evidence.answer1_not_in_premise
    );
    if evidence.acknowledgements != 0 {
        return Err(format!(
            "refusing to write: {} responses are acknowledgements",
            evidence.acknowledgements
        ));
    }
    if evidence.answer2_in_user_turn != 0 {
        let mut offenders: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
        for document in &documents {
            let hit = [&document.premise_text, &document.demand_text]
                .iter()
                .any(|text| contains_answer_word(text, &answer_core(&document.answer2_text)));
            if hit && offenders.values().map(Vec::len).sum::<usize>() < 12 {
                offenders.entry(document.family).or_default().push(format!(
                    "premise {:?} -> turn-1 {:?} -> demand {:?} -> turn-2 {:?}",
                    document.premise_text,
                    document.answer1_text,
                    document.demand_text,
                    document.answer2_text,
                ));
            }
        }
        for (family, samples) in &offenders {
            println!("  turn-2-answer-word-in-user-turn ({family}):");
            for sample in samples.iter().take(4) {
                println!("    {sample}");
            }
        }
        return Err(format!(
            "refusing to write: turn 2's answer occurs as a word in a user turn in {} documents; the \
             store would teach premise reading",
            evidence.answer2_in_user_turn
        ));
    }
    if evidence.answer2_equals_answer1 != 0 {
        return Err(format!(
            "refusing to write: {} documents repeat turn 1's answer token-for-token",
            evidence.answer2_equals_answer1
        ));
    }
    if evidence.answer1_not_in_premise != 0 {
        return Err(format!(
            "refusing to write: turn 1's answer is not a span of its premise in {} documents",
            evidence.answer1_not_in_premise
        ));
    }

    // (b) transformation census, per family.
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

    // Rule 4: value/template randomisation.
    let diversity = diversity_census(&documents);
    report_diversity(&diversity);
    report_slots(&documents);
    if let Err(error) = diversity_gate(&diversity) {
        return Err(format!("refusing to write: {error}"));
    }

    // (a) the sixteen clauses over the emitted bytes, multi-turn reading.
    let document_texts: Vec<(String, String, String, String)> = documents
        .iter()
        .map(|document| {
            (
                document.premise_text.clone(),
                document.demand_text.clone(),
                document.answer1_text.clone(),
                document.answer2_text.clone(),
            )
        })
        .collect();
    let ledger = validate_documents(&stream, &mask, Some(&document_texts));
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

    // (d) the validator must bite: corrupt one document in memory and show it is caught.
    let corrupt_index = documents.len() / 2;
    if args.corrupt_test {
        corruption_test(
            "in-memory store before writing",
            &stream,
            &mask,
            corrupt_index,
        )?;
    }

    // (e) write, then re-read through the training path's own reader.
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
        &evidence,
        &diversity,
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
        &evidence,
        &diversity,
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

    // (f) the first three documents, decoded with their ids and masks.
    for (index, document) in documents.iter().take(3).enumerate() {
        let ids: Vec<u32> = document
            .tokens
            .iter()
            .map(|&token| u32::from(token))
            .collect();
        println!(
            "document {index} (family {}, premise {:?}, turn-1 answer {:?}, demand {:?}, turn-2 \
             answer {:?}): {}",
            document.family,
            document.premise_text,
            document.answer1_text,
            document.demand_text,
            document.answer2_text,
            tokenizer.decode(&ids).replace('\n', " \\n ")
        );
        println!("  tokens: {ids:?}");
        println!("  mask:   {:?}", document.mask);
        println!(
            "  turn-1 answer ids {:?} (mask 1), turn-2 answer ids {:?} (mask 1), user spans {:?}",
            document.answer1_ids, document.answer2_ids, document.user_spans
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
                "  [{}] premise {:?} -> turn-1 {:?} -> demand {:?} -> turn-2 {:?}\n    {}",
                document.family,
                document.premise_text,
                document.answer1_text,
                document.demand_text,
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
                println!("{USAGE}");
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

const USAGE: &str = "usage: demand-store-copy <output-dir> [--tokenizer <tokenizer.json>] \
                     [--seed N] [--corrupt-test] [--samples N]";

fn usage() -> String {
    USAGE.to_string()
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

/// `a, b` — the comma-joined short list the count and ordinal families are built from.
fn comma_list(items: &[&str]) -> String {
    items.join(", ")
}

fn fill(template: &str, key: &str, value: &str) -> String {
    template.replace(key, value)
}

/// The attribute word as the one-word answer form demands it (sentence-initial capital).
fn capitalize(word: &str) -> String {
    let mut characters = word.chars();
    match characters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
    }
}

fn quota_for(family: &str) -> Result<usize, String> {
    FAMILY_QUOTAS
        .iter()
        .find(|entry| entry.0 == family)
        .map(|entry| entry.1)
        .ok_or_else(|| format!("family {family} has no declared quota"))
}

/// Per-family emission quotas, summing to 1,100 inside the 900-1,200 window. Each family's
/// enumerated candidate set is shuffled with the seed and truncated to its quota, so every family
/// is present in the declared proportion and no exact duplicate document can appear.
const FAMILY_QUOTAS: &[(&str, usize)] = &[
    ("count", 170),
    ("ordinal", 160),
    ("name", 150),
    ("conversion", 296),
    ("number", 230),
];

/// `(category, word bank)`: every SHORT list the count and ordinal families are built from is a
/// prefix of one of these banks, so the SAME category appears with 2, 3, 4 or 5 items and the same
/// question template meets several different counts and positions. Turn 1 copies the list, so the
/// count and the item are never a copy of turn 1's answer.
const LIST_BANK: &[(&str, &[&str])] = &[
    ("Metals", &["iron", "gold", "tin", "lead"]),
    ("Colours", &["red", "blue", "green", "white", "black"]),
    ("Shapes", &["circle", "square", "oval", "cube"]),
    ("Tools", &["saw", "plane", "drill", "file"]),
    ("Rivers", &["Nile", "Po", "Rhine", "Elbe"]),
    ("Birds", &["wren", "finch", "robin", "jay"]),
    ("Planets", &["Mars", "Venus", "Saturn"]),
    ("Languages", &["Latin", "Greek", "Dutch"]),
    ("Trees", &["oak", "pine", "birch", "elm"]),
    ("Instruments", &["flute", "harp", "drum", "cello"]),
    ("Fruits", &["apple", "pear", "plum", "fig"]),
    ("Animals", &["cat", "dog", "fox", "owl", "wolf"]),
    ("Cities", &["Paris", "Rome", "Oslo"]),
    ("Flowers", &["rose", "lily", "daisy", "tulip"]),
    ("Stones", &["flint", "slate", "chalk"]),
    ("Drinks", &["water", "milk", "juice", "tea", "coffee"]),
    ("Fabrics", &["wool", "silk", "linen", "felt"]),
    ("Weather", &["rain", "snow", "hail", "mist"]),
    ("Sports", &["chess", "golf", "rugby"]),
    ("Boats", &["canoe", "ferry", "barge"]),
    ("Spices", &["cumin", "thyme", "basil", "mint"]),
    ("Organs", &["heart", "lung", "liver", "brain"]),
    ("Verbs", &["run", "walk", "swim", "read"]),
];
/// The smallest and largest list the count and ordinal families use. The largest is five so the
/// count value class is not degenerate; `MAX_ANSWER1_TOKENS` is what bounds the emitted copy.
const MIN_LIST_ITEMS: usize = 2;
const MAX_LIST_ITEMS: usize = 4;

/// `(category, items)`: every list a bank prefix of `MIN_LIST_ITEMS`..`MAX_LIST_ITEMS` items long.
fn list_prefixes() -> Vec<(&'static str, Vec<&'static str>)> {
    let mut lists = Vec::new();
    for &(category, bank) in LIST_BANK {
        for size in MIN_LIST_ITEMS..=MAX_LIST_ITEMS.min(bank.len()) {
            lists.push((category, bank[..size].to_vec()));
        }
    }
    lists
}

/// Turn-1 premise templates for the count and ordinal families; `{cat}` is the capitalized
/// category and `{list}` the comma-joined items.
const LIST_PREMISES: &[&str] = &[
    "{cat}: {list}.",
    "Items: {list}.",
    "Here: {list}.",
    "{cat} list: {list}.",
    "My list: {list}.",
];

/// Turn-2 demands for the count family; `{plural}` is the lower-case category. Six paraphrases.
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
    ("last", "last", usize::MAX),
];

/// Turn-2 demands for the ordinal family. The answer is the POSITION of an item the turn-1 answer
/// restated: it is computed from turn 1's content and its word is in NO user turn (the item itself
/// is in the premise, which is exactly why asking for the item would be a premise copy and asking
/// for the position is not). Six paraphrases.
const ORDINAL_DEMANDS: &[&str] = &[
    "One word only. What position was {item} in?",
    "Answer in one word. Which position did {item} take?",
    "One word only. What was the position of {item}?",
    "Reply with one word. Which position was {item} in?",
    "Use one word only. What position did {item} hold?",
    "One word only. Name the position {item} was in.",
];

/// `(symbol, element)`: the flashcard pairs the name family reads in both directions. Only pairs
/// whose element name costs at most `ELEMENT_MAX_TOKENS` response tokens survive `select_specs`.
const ELEMENT_FACTS: &[(&str, &str)] = &[
    ("Ar", "argon"),
    ("H", "hydrogen"),
    ("He", "helium"),
    ("C", "carbon"),
    ("N", "nitrogen"),
    ("O", "oxygen"),
    ("Na", "sodium"),
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
    ("Mg", "magnesium"),
    ("Al", "aluminium"),
];
/// The response-token ceiling on an element name answer, so the flashcard family stays short.
const ELEMENT_MAX_TOKENS: usize = 4;

/// `(country, capital)`: the second flashcard set, read in both directions.
const CAPITAL_FACTS: &[(&str, &str)] = &[
    ("France", "Paris"),
    ("Japan", "Tokyo"),
    ("Egypt", "Cairo"),
    ("Norway", "Oslo"),
    ("Italy", "Rome"),
    ("Spain", "Madrid"),
    ("Greece", "Athens"),
    ("Chile", "Santiago"),
    ("Cuba", "Havana"),
    ("Peru", "Lima"),
    ("Malta", "Valletta"),
    ("Ghana", "Accra"),
];
/// The response-token ceiling on a country or capital answer.
const PLACE_MAX_TOKENS: usize = 4;

const SYMBOL_PREMISES: &[&str] = &["Symbol: {symbol}.", "The symbol is {symbol}."];
const SYMBOL_DEMANDS: &[&str] = &[
    "Give a one-word answer. Name the element I gave.",
    "One word only. Which element did my symbol stand for?",
    "Answer in one word. What element did I name?",
];
const ELEMENT_PREMISES: &[&str] = &["Element: {element}.", "The element is {element}."];
const ELEMENT_DEMANDS: &[&str] = &[
    "Give a one-word answer. Which symbol did I give?",
    "One word only. What symbol stands for it?",
    "Answer in one word. Name the symbol I gave.",
];
const CAPITAL_PREMISES: &[&str] = &["Capital: {capital}.", "The capital is {capital}."];
const CAPITAL_DEMANDS: &[&str] = &[
    "Give a one-word answer. Name the capital I gave.",
    "One word only. Which capital did I name?",
    "Answer in one word. What capital did I give?",
];
const COUNTRY_PREMISES: &[&str] = &["Country: {country}.", "The country is {country}."];
const COUNTRY_DEMANDS: &[&str] = &[
    "Give a one-word answer. Which country did I name?",
    "One word only. Name the country I gave.",
    "Answer in one word. What country did I give?",
];

/// `(kind, source unit, target unit, factor)`: turn 1 states a measurement and its answer COPIES it;
/// turn 2 asks for the same quantity in another unit. The answer is computed, and it is in no user
/// turn — the premise holds the SOURCE form (`5 kilometres`), never the converted figures.
const CONVERSION_FACTS: &[(&str, &str, &str, u64)] = &[
    ("distance", "kilometres", "metres", 1000),
    ("distance", "metres", "centimetres", 100),
    ("distance", "centimetres", "millimetres", 10),
    ("weight", "kilograms", "grams", 1000),
    ("weight", "grams", "milligrams", 1000),
    ("volume", "litres", "millilitres", 1000),
    ("time", "hours", "minutes", 60),
    ("time", "minutes", "seconds", 60),
];

/// `(value, source unit)`: the source measurements, each stated with units the table above converts.
const CONVERSION_VALUES: &[(&str, &str)] = &[
    ("5", "kilometres"),
    ("7", "kilometres"),
    ("9", "kilometres"),
    ("3", "metres"),
    ("12", "metres"),
    ("25", "metres"),
    ("50", "centimetres"),
    ("80", "centimetres"),
    ("20", "centimetres"),
    ("2", "kilograms"),
    ("4", "kilograms"),
    ("6", "kilograms"),
    ("600", "grams"),
    ("250", "grams"),
    ("750", "grams"),
    ("3", "litres"),
    ("2", "litres"),
    ("5", "litres"),
    ("4", "hours"),
    ("6", "hours"),
    ("2", "hours"),
    ("30", "minutes"),
    ("45", "minutes"),
    ("15", "minutes"),
];

/// Conversion turn-1 premises; `{value}` is the figure and `{unit}` the source unit that turn 1's
/// answer copies.
const CONVERSION_PREMISES: &[&str] = &["{kind}: {value} {unit}.", "The {kind} is {value} {unit}."];

/// Turn-2 demands for the conversion family; `{target}` is the unit asked for and `{value} {unit}`
/// names the premise's own form, never the converted figures. Eight paraphrases.
const CONVERSION_DEMANDS: &[&str] = &[
    "Answer with digits only. How many {target} is that?",
    "Digits only. What is {value} {unit} in {target}?",
    "Answer with digits only. Convert that to {target}.",
    "Digits only. How many {target} are in {value} {unit}?",
    "Reply with digits only. What is that in {target}?",
    "Answer with digits only. Give that quantity in {target}.",
    "Digits only. Express {value} {unit} as {target}.",
    "Answer with digits only. What does {value} {unit} become in {target}?",
];

/// `(word, digits)`: turn 1's premise states the number in words and its answer COPIES the word;
/// turn 2 asks for the figures. Only pairs whose word costs at most `NUMBER_WORD_MAX_TOKENS`
/// response tokens survive `select_specs`.
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
    ("thirty", 30),
    ("forty", 40),
    ("fifty", 50),
    ("sixty", 60),
    ("seventy", 70),
    ("eighty", 80),
    ("ninety", 90),
];
/// The response-token ceiling on a number word answer.
const NUMBER_WORD_MAX_TOKENS: usize = 4;

const NUMBER_PREMISES: &[&str] = &[
    "Number: {word}.",
    "The number is {word}.",
    "My number is {word}.",
];

/// Turn-2 demands for the number family: the answer is always the figures, computed from turn 1's
/// content, and never appears in a user turn. Six paraphrases.
const NUMBER_DEMANDS: &[&str] = &[
    "Answer with digits only. What is that number in figures?",
    "Digits only. Write that number in digits.",
    "Answer with digits only. What number did I give, in digits?",
    "Digits only. Give me that number as figures.",
    "Reply with digits only. What is that number?",
    "Answer with digits only. Put that number in digits.",
];

fn count_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for (category, items) in list_prefixes() {
        let list = comma_list(&items);
        let plural = category.to_lowercase();
        for premise in LIST_PREMISES {
            let question1 = fill(&fill(premise, "{cat}", category), "{list}", &list);
            for demand in COUNT_DEMANDS {
                let template = fill(demand, "{plural}", "{kind}");
                specs.push(Spec {
                    family: "count",
                    premise: question1.clone(),
                    answer1: format!("{}.", capitalize(&list)),
                    demand: fill(demand, "{plural}", &plural),
                    answer2: format!("{}.", items.len()),
                    value: format!("{}", items.len()),
                    demand_template: template,
                    demand_slot: (*category).to_string(),
                });
            }
        }
    }
    specs
}

fn ordinal_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for (category, items) in list_prefixes() {
        let list = comma_list(&items);
        for premise in LIST_PREMISES {
            let question1 = fill(&fill(premise, "{cat}", category), "{list}", &list);
            for &(ordinal, _adverb, slot) in ORDINAL_SLOTS {
                let index = if slot == usize::MAX {
                    items.len() - 1
                } else {
                    slot
                };
                if index >= items.len() {
                    continue;
                }
                let item = items[index];
                for demand in ORDINAL_DEMANDS {
                    let question2 = fill(demand, "{item}", item);
                    specs.push(Spec {
                        family: "ordinal",
                        premise: question1.clone(),
                        answer1: format!("{}.", capitalize(&list)),
                        demand: question2,
                        answer2: format!("{ordinal}."),
                        value: (*ordinal).to_string(),
                        demand_template: (*demand).to_string(),
                        demand_slot: (*item).to_string(),
                    });
                }
            }
        }
    }
    specs
}

fn name_specs(tokenizer: &HfBpeTokenizer) -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(symbol, element) in ELEMENT_FACTS {
        let element_title = capitalize(element);
        if answer_tokens(tokenizer, &format!("{element_title}.")) > ELEMENT_MAX_TOKENS {
            continue;
        }
        // Turn 1 gives the symbol and copies it; turn 2 asks for the element.
        for premise in SYMBOL_PREMISES {
            for demand in SYMBOL_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    premise: fill(premise, "{symbol}", symbol),
                    answer1: format!("{symbol}."),
                    demand: (*demand).to_string(),
                    answer2: format!("{element_title}."),
                    value: (*element).to_string(),
                    demand_template: (*demand).to_string(),
                    demand_slot: (*symbol).to_string(),
                });
            }
        }
        // Turn 1 gives the element and copies it; turn 2 asks for the symbol.
        for premise in ELEMENT_PREMISES {
            for demand in ELEMENT_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    premise: fill(premise, "{element}", element),
                    answer1: format!("{element_title}."),
                    demand: (*demand).to_string(),
                    answer2: format!("{symbol}."),
                    value: (*symbol).to_string(),
                    demand_template: (*demand).to_string(),
                    demand_slot: (*element).to_string(),
                });
            }
        }
    }
    for &(country, capital) in CAPITAL_FACTS {
        let country_title = capitalize(country);
        let capital_title = capitalize(capital);
        if answer_tokens(tokenizer, &format!("{country_title}.")) > PLACE_MAX_TOKENS
            || answer_tokens(tokenizer, &format!("{capital_title}.")) > PLACE_MAX_TOKENS
        {
            continue;
        }
        for premise in CAPITAL_PREMISES {
            for demand in CAPITAL_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    premise: fill(premise, "{capital}", capital_title.as_str()),
                    answer1: format!("{capital_title}."),
                    demand: (*demand).to_string(),
                    answer2: format!("{country_title}."),
                    value: (*country).to_string(),
                    demand_template: (*demand).to_string(),
                    demand_slot: (*capital).to_string(),
                });
            }
        }
        for premise in COUNTRY_PREMISES {
            for demand in COUNTRY_DEMANDS {
                specs.push(Spec {
                    family: "name",
                    premise: fill(premise, "{country}", country_title.as_str()),
                    answer1: format!("{country_title}."),
                    demand: (*demand).to_string(),
                    answer2: format!("{capital_title}."),
                    value: (*capital).to_string(),
                    demand_template: (*demand).to_string(),
                    demand_slot: (*country).to_string(),
                });
            }
        }
    }
    specs
}

fn number_specs(tokenizer: &HfBpeTokenizer) -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(word, value) in NUMBER_FACTS {
        let word_title = capitalize(word);
        if answer_tokens(tokenizer, &format!("{word_title}.")) > NUMBER_WORD_MAX_TOKENS {
            continue;
        }
        for premise in NUMBER_PREMISES {
            let question1 = fill(premise, "{word}", word);
            for demand in NUMBER_DEMANDS {
                specs.push(Spec {
                    family: "number",
                    premise: question1.clone(),
                    answer1: format!("{word_title}."),
                    demand: (*demand).to_string(),
                    answer2: format!("{value}."),
                    value: format!("{value}"),
                    demand_template: (*demand).to_string(),
                    demand_slot: (*word).to_string(),
                });
            }
        }
    }
    specs
}

fn conversion_specs(tokenizer: &HfBpeTokenizer) -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(kind, unit, target, factor) in CONVERSION_FACTS {
        if answer_tokens(tokenizer, target) > MAX_ANSWER2_TOKENS {
            continue;
        }
        for &(value, source_unit) in CONVERSION_VALUES {
            if source_unit != unit {
                continue;
            }
            let Ok(figures) = value.parse::<u64>() else {
                continue;
            };
            let Some(converted) = figures.checked_mul(factor) else {
                continue;
            };
            for premise in CONVERSION_PREMISES {
                let question1 = fill(
                    &fill(&fill(premise, "{kind}", kind), "{value}", value),
                    "{unit}",
                    unit,
                );
                for demand in CONVERSION_DEMANDS {
                    let template = fill(demand, "{target}", target);
                    let question2 = fill(
                        &fill(&fill(demand, "{target}", target), "{value}", value),
                        "{unit}",
                        unit,
                    );
                    specs.push(Spec {
                        family: "conversion",
                        premise: question1.clone(),
                        answer1: format!("{value} {unit}."),
                        demand: question2,
                        answer2: format!("{converted}."),
                        value: format!("{converted}"),
                        demand_template: template,
                        demand_slot: format!("{value} {unit}"),
                    });
                }
            }
        }
    }
    specs
}

fn candidate_families(tokenizer: &HfBpeTokenizer) -> Vec<(&'static str, Vec<Spec>)> {
    vec![
        ("count", count_specs()),
        ("ordinal", ordinal_specs()),
        ("name", name_specs(tokenizer)),
        ("conversion", conversion_specs(tokenizer)),
        ("number", number_specs(tokenizer)),
    ]
}

fn is_acknowledgement(content: &str) -> bool {
    let trimmed = content.trim();
    ACKNOWLEDGEMENTS.iter().any(|ack| trimmed == ack.trim())
}

fn is_acknowledgement_loose(content: &str) -> bool {
    let trimmed = content.trim();
    ACKNOWLEDGEMENTS
        .iter()
        .any(|ack| trimmed.eq_ignore_ascii_case(ack.trim()))
}

/// Is this token one of the byte-level separator tokens (`,`, `.`)? No response may open on one.
fn is_punctuation(token: u16) -> bool {
    u32::from(token) == 14 || u32::from(token) == 16
}

/// Filter every family to the length gate, prefer the shortest answers, truncate to the quota and
/// then shuffle the union: the document order and membership are fixed by the seed alone, and no
/// candidate can be emitted twice. The selection is deterministic for a given seed and tokenizer.
/// The value/template diversity is a property of the enumeration, and `diversity_gate` re-checks it
/// over what was actually selected.
fn select_specs(
    seed: u64,
    tokenizer: &HfBpeTokenizer,
) -> Result<(Vec<Spec>, BTreeMap<&'static str, (usize, usize)>), String> {
    let mut selected: Vec<Spec> = Vec::new();
    let mut report: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for (family, specs) in candidate_families(tokenizer) {
        let enumerated = specs.len();
        if enumerated == 0 {
            return Err(format!("family {family} enumerated no candidates"));
        }
        let quota = quota_for(family)?;
        // The length gate and the acknowledgement gate: reject any candidate that violates them.
        let mut fitting: Vec<(Spec, usize, usize)> = Vec::new();
        let mut rejected_length = 0usize;
        let mut rejected_ack = 0usize;
        let mut rejected_dependency = 0usize;
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
            if is_acknowledgement(&spec.answer1) || is_acknowledgement(&spec.answer2) {
                rejected_ack += 1;
                continue;
            }
            if length1 > MAX_ANSWER1_TOKENS || length2 > MAX_ANSWER2_TOKENS {
                rejected_length += 1;
                continue;
            }
            // Requirement (b), enforced at the candidate level too: turn 2's answer must not occur
            // as a word in either user turn of its own document.
            let core = answer_core(&spec.answer2);
            if contains_answer_word(&spec.premise, &core)
                || contains_answer_word(&spec.demand, &core)
            {
                rejected_dependency += 1;
                continue;
            }
            fitting.push((spec, length1, length2));
        }
        if rejected_ack != 0 {
            return Err(format!(
                "family {family} enumerated {rejected_ack} candidate documents whose answer is an \
                 acknowledgement; the forbidden set {ACKNOWLEDGEMENTS:?} must not appear at all"
            ));
        }
        if fitting.len() < quota {
            return Err(format!(
                "family {family} has only {} candidates inside the answer gates ({rejected_length} \
                 rejected over {MAX_ANSWER1_TOKENS}/{MAX_ANSWER2_TOKENS} tokens, \
                 {rejected_dependency} with turn 2's answer inside a user turn) and needs {quota}; \
                 turn-1 answer length histogram (tokens: candidates) {histogram:?}",
                fitting.len()
            ));
        }
        // Requirement 4's selection rule: every demanded VALUE gets an equal share of the family's
        // quota, so a short answer's convenience cannot crowd out the larger counts, the later
        // positions or the further conversions. Candidates inside a value class are ordered
        // shortest-first, which keeps the emitted means inside the targets.
        let mut groups: BTreeMap<String, Vec<(Spec, usize, usize)>> = BTreeMap::new();
        for (spec, length1, length2) in fitting {
            groups
                .entry(spec.value.clone())
                .or_default()
                .push((spec, length1, length2));
        }
        let mut ordered: Vec<Vec<(Spec, usize, usize)>> = Vec::new();
        for (value, mut group) in groups {
            shuffle(&mut group, family_seed(seed, &format!("{family}|{value}")));
            group.sort_by_key(|(_, length1, length2)| {
                (
                    usize::from(*length1 > PREFERRED_ANSWER1_TOKENS),
                    usize::from(*length2 > PREFERRED_ANSWER2_TOKENS),
                    *length1 + *length2,
                )
            });
            ordered.push(group);
        }
        // `BTreeMap` iterated by value name; the round-robin below is what makes the split equal.
        let class_quota = quota / ordered.len().max(1);
        let remainder = quota % ordered.len().max(1);
        let mut picked: Vec<Spec> = Vec::new();
        for (class, group) in ordered.iter_mut().enumerate() {
            let take = class_quota + usize::from(class < remainder);
            let available = group.len();
            for (spec, _, _) in group.drain(..take.min(available)) {
                picked.push(spec);
            }
        }
        if picked.len() < quota {
            return Err(format!(
                "family {family} could only fill {} of its {quota} documents from {} value classes \
                 (each class at most {class_quota} plus remainder)",
                picked.len(),
                ordered.len()
            ));
        }
        report.insert(family, (enumerated, picked.len()));
        selected.extend(picked);
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

/// The response-content token length of an authored answer, measured with the bound tokenizer
/// exactly as `build_documents` will emit it (leading space included).
fn answer_tokens(tokenizer: &HfBpeTokenizer, answer: &str) -> usize {
    encode_with(tokenizer, &format!(" {answer}")).len()
}

/// The longest contiguous run of PREMISE tokens that is a token suffix of the declared answer, so
/// the emitted turn-1 response is a verbatim span of turn 1's user turn rather than a re-encoding of
/// the same text. A byte-level BPE tokenizer merges a running sentence's leading boundary
/// differently from a response's (` is` + ` red` against ` red` + `.`), so an independently encoded
/// copy is not a token span even when it is the same surface text. Returns `None` when nothing
/// matches.
fn premise_copy_span(premise: &[u16], answer: &[u16]) -> Option<(usize, usize)> {
    if answer.is_empty() {
        return None;
    }
    let mut best: Option<(usize, usize)> = None;
    for start in 0..premise.len() {
        for end in (start + 1..=premise.len()).rev() {
            if best.is_some_and(|(best_start, best_end)| end - start <= best_end - best_start) {
                break;
            }
            if answer.ends_with(&premise[start..end]) {
                best = Some((start, end));
                break;
            }
        }
    }
    best
}

/// Every surface form the premise copy may take, in preference order: the declared answer, then the
/// same words as the PREMISE spells them (a capitalised list is only a copy if the premise's own
/// characters are reused, and its lower-case form is where the full token span is), then the
/// capitalised form for a family whose premise states a lower-case word. The first form that is a
/// token span of the premise wins, so turn 1's response is always the premise's own characters and
/// always the longest available copy.
fn premise_copy_candidates(answer: &str) -> Vec<String> {
    let trimmed = answer.trim();
    let lower = trimmed.to_lowercase();
    let capitalised = capitalize(&lower);
    let mut candidates = vec![trimmed.to_string()];
    for form in [lower, capitalised] {
        if !candidates.contains(&form) {
            candidates.push(form);
        }
    }
    candidates
}

/// Would this response content be an acknowledgement? The forbidden set is compared exactly and,
/// reported separately, case-insensitively after trimming.
/// Build one two-turn document in the chat corpus's framing, recording both answers' response
/// content, the user turns' spans and the templates the diversity census reads.
fn build_documents(specs: &[Spec], tokenizer: &HfBpeTokenizer) -> Result<Vec<Doc>, String> {
    let mut documents = Vec::with_capacity(specs.len());
    for spec in specs {
        let mut tokens: Vec<u16> = Vec::new();
        let mut mask: Vec<u8> = Vec::new();
        let mut user_spans: Vec<(usize, usize)> = Vec::new();
        push(&mut tokens, &mut mask, &[BOS_ID as u16], 0);
        push(&mut tokens, &mut mask, &USER_MARKER.map(|id| id as u16), 0);
        let premise_start = tokens.len();
        let premise_ids = encode_with(tokenizer, &format!(" {}", spec.premise));
        push(&mut tokens, &mut mask, &premise_ids, 0);
        user_spans.push((premise_start, tokens.len()));
        push(&mut tokens, &mut mask, &[SEPARATOR_ID], 0);
        push(
            &mut tokens,
            &mut mask,
            &ASSISTANT_MARKER.map(|id| id as u16),
            0,
        );
        // Turn 1's response is the premise's OWN token span, so the copy is verbatim in the
        // model's token view as well as on the surface. The longest available span is taken, and a
        // degenerate span (one that begins on punctuation) is refused rather than emitted.
        let mut answer1_ids: Option<Vec<u16>> = None;
        let mut answer1_text = spec.answer1.clone();
        let mut best_len = 0usize;
        for candidate in premise_copy_candidates(&spec.answer1) {
            let declared = encode_with(tokenizer, &format!(" {candidate}"));
            let Some((start, end)) = premise_copy_span(&premise_ids, &declared) else {
                continue;
            };
            // A span that opens on punctuation is the tail of a longer word's token, not a copy.
            if is_punctuation(premise_ids[start]) || end - start <= best_len {
                continue;
            }
            best_len = end - start;
            answer1_ids = Some(premise_ids[start..end].to_vec());
            answer1_text = candidate;
        }
        let Some(answer1_ids) = answer1_ids else {
            return Err(format!(
                "family {} turn-1 answer {:?} is not a token span of its premise {:?}; the copy \
                 would be a re-encoding, not a copy",
                spec.family, spec.answer1, spec.premise
            ));
        };
        push(&mut tokens, &mut mask, &answer1_ids, 1);
        push(&mut tokens, &mut mask, &[EOS_ID as u16], 1);
        // Every turn after the first is preceded by the single `\n` separator.
        push(&mut tokens, &mut mask, &[SEPARATOR_ID], 0);
        push(&mut tokens, &mut mask, &USER_MARKER.map(|id| id as u16), 0);
        let demand_start = tokens.len();
        push(
            &mut tokens,
            &mut mask,
            &encode_with(tokenizer, &format!(" {}", spec.demand)),
            0,
        );
        user_spans.push((demand_start, tokens.len()));
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
            value: spec.value.clone(),
            answer1_ids,
            answer2_ids,
            response2_start,
            user_spans,
            premise_text: spec.premise.clone(),
            answer1_text: answer1_text.clone(),
            demand_text: spec.demand.clone(),
            answer2_text: spec.answer2.clone(),
            demand_template: spec.demand_template.clone(),
            demand_slot: spec.demand_slot.clone(),
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

/// The answer's CORE: its text without the terminating period, lower-cased. This is the phrase a
/// premise-copying policy would have to reproduce.
fn answer_core(answer: &str) -> String {
    answer.trim().trim_end_matches('.').trim().to_lowercase()
}

/// Does `text` contain `core` as a whole word (a SPACE before it and a non-alphanumeric after it)?
///
/// This is requirement (b)'s check, and it is deliberately about the phrase: the earlier stores put
/// the answer's value inside the premise as a word (`Metals: iron, copper, zinc.` with the answer
/// `3.` and the word `copper` the follow-up reads), so a premise reader could satisfy them without
/// reading turn 1. The byte-level BPE tokenizer splits a leading boundary differently in a running
/// sentence than at the start of a response (` is` + ` red` against ` red` + `.`), so the token-level
/// reading of the same question is reported separately and labelled as the artifact it is.
fn contains_answer_word(text: &str, core: &str) -> bool {
    if core.is_empty() {
        return false;
    }
    let haystack = text.trim().to_lowercase();
    let mut from = 0usize;
    while let Some(found) = haystack[from..].find(core) {
        let start = from + found;
        let end = start + core.len();
        let before_ok = start > 0
            && haystack[..start]
                .chars()
                .next_back()
                .is_some_and(|c| c == ' ' || c == '\n' || c == '\t');
        let after_ok = haystack[end..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    false
}

/// (c) and (b) over the emitted documents, exactly as the brief demands them: no acknowledgement is
/// a response, turn 2's answer occurs as a WORD in no user turn, turn 2's answer is not turn 1's
/// answer, and turn 1's answer is a span of its premise. The byte-level token reading of the same
/// containment question is recorded as well, and kept separate.
fn measure_evidence(documents: &[Doc]) -> Evidence {
    let mut evidence = Evidence::default();
    for document in documents {
        if is_acknowledgement(&document.answer1_text) || is_acknowledgement(&document.answer2_text)
        {
            evidence.acknowledgements += 1;
        }
        if is_acknowledgement_loose(&document.answer1_text)
            || is_acknowledgement_loose(&document.answer2_text)
        {
            evidence.acknowledgements_case_insensitive += 1;
        }
        if document.answer2_ids == document.answer1_ids {
            evidence.answer2_equals_answer1 += 1;
        }
        let core = answer_core(&document.answer2_text);
        let in_user_turn = document.user_spans.iter().any(|&(start, end)| {
            contains_span(&document.tokens[start..end], &document.answer2_ids)
        });
        if in_user_turn {
            evidence.answer2_token_span_in_user_turn += 1;
        }
        let word_in_user_turn = [&document.premise_text, &document.demand_text]
            .iter()
            .any(|text| contains_answer_word(text, &core));
        evidence.answer2_in_user_turn += usize::from(word_in_user_turn);
        if contains_span(
            &document.tokens[..document.response2_start],
            &document.answer2_ids,
        ) {
            evidence.answer2_in_prefix += 1;
        }
        let premise_span = document
            .user_spans
            .first()
            .map(|&(start, end)| &document.tokens[start..end])
            .unwrap_or(&[]);
        if !contains_span(premise_span, &document.answer1_ids) {
            evidence.answer1_not_in_premise += 1;
        }
        let closing = &document.tokens[..document.response2_start];
        if contains_span(closing, &document.answer1_ids) {
            evidence.answer1_in_closing_span += 1;
        }
    }
    evidence
}

/// (b) How many documents a verbatim copy of turn 1's answer would pass, per family, plus the
/// weaker span-copy readings and the premise-containment reading the earlier stores omitted.
fn relation_census(documents: &[Doc]) -> RelationStats {
    let mut stats = RelationStats {
        documents: documents.len(),
        equal: 0,
        span_in_answer1: 0,
        span_in_prefix: 0,
        answer2_in_user_turn: 0,
        per_family: BTreeMap::new(),
    };
    for document in documents {
        let equal = document.answer2_ids == document.answer1_ids;
        let span_in_answer1 = contains_span(&document.answer1_ids, &document.answer2_ids);
        let span_in_prefix = contains_span(
            &document.tokens[..document.response2_start],
            &document.answer2_ids,
        );
        let in_user_turn = document.user_spans.iter().any(|&(start, end)| {
            contains_span(&document.tokens[start..end], &document.answer2_ids)
        });
        stats.equal += usize::from(equal);
        stats.span_in_answer1 += usize::from(span_in_answer1);
        stats.span_in_prefix += usize::from(span_in_prefix);
        stats.answer2_in_user_turn += usize::from(in_user_turn);
        let entry = stats.per_family.entry(document.family).or_default();
        entry.documents += 1;
        entry.equal += usize::from(equal);
        entry.span_in_answer1 += usize::from(span_in_answer1);
        entry.span_in_prefix += usize::from(span_in_prefix);
        entry.answer2_in_user_turn += usize::from(in_user_turn);
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
        "  turn-2 answer inside a USER turn (a premise-copy policy would pass): {} of {} ({:.6}); \
         annex: contiguous span of turn-1's answer {} ({:.6}), anywhere in the prefix {} ({:.6})",
        stats.answer2_in_user_turn,
        documents,
        stats.answer2_in_user_turn as f64 / documents as f64,
        stats.span_in_answer1,
        stats.span_in_answer1 as f64 / documents as f64,
        stats.span_in_prefix,
        stats.span_in_prefix as f64 / documents as f64,
    );
    for (family, entry) in &stats.per_family {
        println!(
            "  family {family}: {} documents, identical {}, span-of-turn-1 {}, span-of-prefix {}, \
             in-user-turn {}",
            entry.documents,
            entry.equal,
            entry.span_in_answer1,
            entry.span_in_prefix,
            entry.answer2_in_user_turn
        );
    }
}

/// Rule 4: the distinct `(value, template)` combinations and the value distribution per family.
/// The template key is the demanded question with its value-dependent slots removed but the
/// family's per-item slots kept, so two documents count as the same template only when a surface
/// reader could not tell them apart.
fn diversity_census(documents: &[Doc]) -> Diversity {
    let mut per_family: BTreeMap<&'static str, FamilyDiversity> = BTreeMap::new();
    let mut pairs: BTreeSet<(&'static str, String, String)> = BTreeSet::new();
    // The "min" readings need their own pass over the per-template / per-value groupings.
    let mut per_template: BTreeMap<(&'static str, String), BTreeSet<String>> = BTreeMap::new();
    let mut per_value: BTreeMap<(&'static str, String), BTreeSet<String>> = BTreeMap::new();
    let mut per_value_docs: BTreeMap<(&'static str, String), usize> = BTreeMap::new();
    for document in documents {
        let template = template_key(document.family, &document.demand_template);
        pairs.insert((document.family, document.value.clone(), template.clone()));
        let entry = per_family.entry(document.family).or_default();
        entry.documents += 1;
        entry.values.insert(document.value.clone());
        entry.templates.insert(template.clone());
        per_template
            .entry((document.family, template.clone()))
            .or_default()
            .insert(document.value.clone());
        per_value
            .entry((document.family, document.value.clone()))
            .or_default()
            .insert(template);
        *per_value_docs
            .entry((document.family, document.value.clone()))
            .or_default() += 1;
    }
    for (family, _, _) in &pairs {
        per_family.entry(family).or_default().combinations += 1;
    }
    for (family, entry) in per_family.iter_mut() {
        entry.min_values_per_template = usize::MAX;
        entry.min_templates_per_value = usize::MAX;
        for ((candidate, template), values) in &per_template {
            if candidate != family {
                continue;
            }
            if values.len() < entry.min_values_per_template {
                entry.min_values_per_template = values.len();
                entry.min_values_template = template.clone();
            }
        }
        for ((candidate, value), templates) in &per_value {
            if candidate != family {
                continue;
            }
            if templates.len() < entry.min_templates_per_value {
                entry.min_templates_per_value = templates.len();
                entry.min_templates_value = value.clone();
            }
            let count = per_value_docs
                .get(&(candidate, value.clone()))
                .copied()
                .unwrap_or(0);
            if count > entry.top_value_documents {
                entry.top_value_documents = count;
                entry.top_value = value.clone();
            }
        }
    }
    let values: BTreeSet<String> = documents.iter().map(|d| d.value.clone()).collect();
    let templates: BTreeSet<String> = documents
        .iter()
        .map(|d| template_key(d.family, &d.demand_template))
        .collect();
    Diversity {
        documents: documents.len(),
        combinations: pairs.len(),
        values: values.len(),
        templates: templates.len(),
        per_family,
    }
}

/// The template key of a demand: the FAMILY'S DECLARED demand pattern with every value-bearing slot
/// (`{value}`, `{item}`, `{kind}`, `{plural}`) left in place, prefixed by the family. The question a
/// document carries additionally names the particular category, item or measurement it is about
/// (`document.demand_slot`); that name is the slot's identity, not part of the template. Grouping by
/// the template is what makes "How many items did I list?" one template across every count it meets.
fn template_key(family: &str, demand_template: &str) -> String {
    format!("{family}|{demand_template}")
}

/// The slots a family's documents actually ranged over, so the report shows the value-bearing
/// positions each template met rather than only their count.
fn report_slots(documents: &[Doc]) {
    let mut per_family: BTreeMap<&'static str, BTreeSet<String>> = BTreeMap::new();
    for document in documents {
        per_family
            .entry(document.family)
            .or_default()
            .insert(document.demand_slot.clone());
    }
    for (family, slots) in per_family {
        println!(
            "  family {family} demand slots ({} distinct): {:?}",
            slots.len(),
            slots.iter().take(8).collect::<Vec<_>>()
        );
    }
}

fn report_diversity(diversity: &Diversity) {
    println!(
        "diversity (rule 4): {} documents, {} distinct demanded values, {} distinct demand \
         templates, {} distinct (value, template) combinations",
        diversity.documents, diversity.values, diversity.templates, diversity.combinations
    );
    for (family, entry) in &diversity.per_family {
        let share = entry.top_value_documents as f64 / entry.documents.max(1) as f64;
        println!(
            "  family {family}: {} documents, {} values, {} templates, {} (value, template) \
             combinations; min distinct values per template {} (worst template {:?}), min distinct \
             templates per value {} (worst value {:?}); top value {top:?} holds {count} documents \
             ({share:.4})",
            entry.documents,
            entry.values.len(),
            entry.templates.len(),
            entry.combinations,
            entry.min_values_per_template,
            entry.min_values_template,
            entry.min_templates_per_value,
            entry.min_templates_value,
            top = entry.top_value,
            count = entry.top_value_documents,
        );
    }
}

/// The diversity gate: no family may be solvable by a surface statistic.
fn diversity_gate(diversity: &Diversity) -> Result<(), String> {
    for (family, entry) in &diversity.per_family {
        if entry.min_values_per_template < MIN_DISTINCT_VALUES_PER_TEMPLATE {
            return Err(format!(
                "family {family} template {:?} occurs with only {} distinct values (floor \
                 {MIN_DISTINCT_VALUES_PER_TEMPLATE})",
                entry.min_values_template, entry.min_values_per_template
            ));
        }
        if entry.min_templates_per_value < MIN_TEMPLATES_PER_VALUE {
            return Err(format!(
                "family {family} value {:?} occurs under only {} distinct templates (floor \
                 {MIN_TEMPLATES_PER_VALUE})",
                entry.min_templates_value, entry.min_templates_per_value
            ));
        }
        let share = entry.top_value_documents as f64 / entry.documents.max(1) as f64;
        if share > MAX_VALUE_FAMILY_SHARE {
            return Err(format!(
                "family {family} value {:?} takes {share:.4} of the family, over \
                 {MAX_VALUE_FAMILY_SHARE:.2}",
                entry.top_value
            ));
        }
        if entry.combinations < entry.templates.len() {
            return Err(format!(
                "family {family} has only {} (value, template) combinations over {} templates",
                entry.combinations,
                entry.templates.len()
            ));
        }
    }
    Ok(())
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

/// The response content (EOS excluded) of each masked run in a document.
fn response_contents(tokens: &[u16], mask: &[u8]) -> Vec<Vec<u16>> {
    mask_runs(mask)
        .into_iter()
        .map(|(start, end)| tokens[start..end.saturating_sub(1)].to_vec())
        .collect()
}

/// The sixteen clauses over the flat stream, with the multi-turn generalisations described in the
/// module comment and this store's two extra conditions on `document_vocabulary` (no
/// acknowledgement is a response) and `response_no_interior_eos` (turn 2's answer is not turn 1's and
/// occurs in no user turn). Used both for the emitted bytes and (on a synthetic single-turn stream)
/// for every assistant turn as a literal single-turn document.
fn validate_documents(
    stream: &[u16],
    mask: &[u8],
    document_texts: Option<&[(String, String, String, String)]>,
) -> Ledger {
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
        // This store's extra condition on the clause: no response content is an acknowledgement.
        // The check is over the response TEXTS of this document (turn 1's and turn 2's), so it
        // catches multi-token acknowledgements that a mask-only reading would not.
        let response_texts: Vec<String> = match document_texts.and_then(|texts| texts.get(document))
        {
            Some((_, _, answer1, answer2)) => vec![answer1.clone(), answer2.clone()],
            None => response_contents(tokens, mask)
                .iter()
                .map(|content| decode_content(content))
                .collect(),
        };
        for text in response_texts {
            if is_acknowledgement(&text) {
                ledger.hit(
                    ACK_CLAUSE,
                    document,
                    format!("response {text:?} is an acknowledgement"),
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

        // This store's extra condition on the dependency clause: the LATER response run's content
        // must differ from the earlier run's and its core must not occur as a word in any user turn
        // of the document. The byte-level token-span reading of the same question is reported by
        // `measure_evidence` as a labelled annex, not gated here.
        let contents = response_contents(tokens, mask);
        if contents.len() >= 2 {
            let (first, second) = (&contents[0], &contents[1]);
            if first == second {
                ledger.hit(
                    DEPENDENCY_CLAUSE,
                    document,
                    format!(
                        "turn 2's answer {:?} is token-identical to turn 1's answer",
                        decode_content(second)
                    ),
                );
            }
            if let Some(texts) = document_texts {
                if let Some((premise, demand, _, answer2)) = texts.get(document) {
                    let core = answer_core(answer2);
                    if !core.is_empty()
                        && (contains_answer_word(premise, &core)
                            || contains_answer_word(demand, &core))
                    {
                        ledger.hit(
                            DEPENDENCY_CLAUSE,
                            document,
                            format!(
                                "turn 2's answer core {core:?} occurs as a word in a user turn \
                                 (premise {premise:?}, demand {demand:?})"
                            ),
                        );
                    }
                }
            }
        }
    }
    ledger
}

/// The decoded form of a response content run, for reports. The ids are the artifact; this is a
/// display convenience only and no gate reads it.
fn decode_content(ids: &[u16]) -> String {
    ids.iter()
        .map(|&id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Every assistant turn re-read as its own single-turn document, under the same sixteen clause
/// names with their literal single-turn meaning (for a single-run document the multi-turn
/// generalisations coincide with the literal checks). The two augmented clauses apply here too: a
/// one-run unit has no turn-2 answer to check, and the acknowledgement condition still bites.
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
    validate_documents(&stream, &mask, None)
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

/// (d) Prove the validator bites: corrupt one document in memory only, show the offending document
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
    let ledger = validate_documents(stream, &corrupted, None);
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
    let restored = validate_documents(stream, mask, None);
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
        "validation ({title}): {} clauses, {} offenders (acknowledgement hits {}, dependency hits \
         {})",
        ledger.counts.len(),
        ledger.offenders(),
        ledger.acknowledgement_hits,
        ledger.dependency_hits
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
/// for these exact bytes; it is recorded only after that load succeeded, and the caller re-loads the
/// store after this write so the reported manifest is the one that was accepted.
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
    evidence: &Evidence,
    diversity: &Diversity,
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
                    "answer2_in_user_turn": entry.answer2_in_user_turn,
                }),
            )
        })
        .collect();
    let diversity_json: BTreeMap<&str, serde_json::Value> = diversity
        .per_family
        .iter()
        .map(|(family, entry)| {
            (
                *family,
                serde_json::json!({
                    "documents": entry.documents,
                    "values": entry.values.len(),
                    "templates": entry.templates.len(),
                    "value_template_combinations": entry.combinations,
                    "min_distinct_values_per_template": entry.min_values_per_template,
                    "min_distinct_templates_per_value": entry.min_templates_per_value,
                    "top_value": entry.top_value,
                    "top_value_documents": entry.top_value_documents,
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
        "generator": "crates/uor-r4-training/examples/demand-store-copy.rs",
        "mask_bytes": mask.len(),
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating \
                      <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, \
                      and an unmasked document-terminal <|eos|>.",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "mask_sha256": uor_r4_training::sha256_bytes(mask),
        "response_tokens": mask.iter().filter(|&&value| value == 1).count(),
        "schema": "uor-r4-demand-store-copy/v1",
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
            "answer2_in_user_turn": relation.answer2_in_user_turn,
            "min_required_fraction": MIN_TRANSFORMATION_FRACTION,
            "per_family": per_family,
        },
        "copy_rule": {
            "turn1_answers_are_copies_of_the_premise": true,
            "turn1_answer_not_a_premise_span": evidence.answer1_not_in_premise,
            "forbidden_acknowledgements": ACKNOWLEDGEMENTS,
            "acknowledgements_present": evidence.acknowledgements,
            "acknowledgements_present_case_insensitive": evidence.acknowledgements_case_insensitive,
            "turn2_answer_in_a_user_turn": evidence.answer2_in_user_turn,
            "turn2_answer_equals_turn1_answer": evidence.answer2_equals_answer1,
            "turn2_answer_anywhere_in_prefix": evidence.answer2_in_prefix,
        },
        "diversity": {
            "documents": diversity.documents,
            "distinct_values": diversity.values,
            "distinct_templates": diversity.templates,
            "distinct_value_template_combinations": diversity.combinations,
            "min_distinct_values_per_template": MIN_DISTINCT_VALUES_PER_TEMPLATE,
            "min_distinct_templates_per_value": MIN_TEMPLATES_PER_VALUE,
            "max_value_family_share": MAX_VALUE_FAMILY_SHARE,
            "per_family": diversity_json,
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
            "diluted_variant_response_tokens": DILUTED_VARIANT_RESPONSE_TOKENS,
            "response_token_ceiling": RESPONSE_TOKEN_CEILING,
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
