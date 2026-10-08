//! Build an ATTRIBUTE-EXTRACTION demand store.
//!
//! ```text
//! demand-store-attr <output-dir> [--tokenizer <tokenizer.json>] [--seed N] [--corrupt-test]
//!                   [--samples N]
//! ```
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
//! Families (every emitted document belongs to exactly one):
//!
//! - `count`:     t1 lists items, t2 asks `How many items were in your previous answer?` -> `3.`
//! - `ordinal`:   t1 lists items, t2 asks `What was the second item in that list?` -> `orange.`
//! - `name`:      t1 `The capital of France is Paris.`, t2 asks for one word -> `Paris.`
//! - `number`:    t1 `Ten plus ten is 20.`, t2 asks `That number minus one?` -> `19.`
//! - `polarity`:  t1 `Yes, the sky is blue.`, t2 asks about the previous answer -> `No.`
//! - `flip_word`: t1 `Yes.`, t2 asks `Was your previous answer no?` -> `No.`
//! - `attribute`: t1 `The rose is red.`, t2 asks one word -> `Red.`
//!
//! Every turn-1 answer in the attribute-bearing families is a sentence, so the short turn-2 answer
//! cannot be a verbatim copy of it; the census refuses to write when any second answer equals its
//! first answer token-for-token.

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
const OUTPUT_LABEL: &str = "demand-attr";
/// This tool's document-order seed; `--seed` overrides it.
const DEFAULT_SEED: u64 = 0x0000_6174_7472_6962; // "attrib"
/// How many offending documents the refusal prints before summarizing.
const MAX_REPORTED_OFFENDERS: usize = 20;
/// The demanded floor on the fraction of documents whose second answer is not a verbatim copy of
/// the first answer's token sequence.
const MIN_TRANSFORMATION_FRACTION: f64 = 0.80;

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

/// Per-family emission quotas: each family's enumerated candidate set is shuffled with the seed and
/// truncated to its quota, so every family is present in the declared proportion and no exact
/// duplicate document can appear.
const FAMILY_QUOTAS: &[(&str, usize)] = &[
    ("count", 640),
    ("ordinal", 600),
    ("name", 480),
    ("number", 240),
    ("polarity", 200),
    ("flip_word", 40),
    ("attribute", 400),
];

/// `(category, items)`: the item lists the count/ordinal families read a number or a position out
/// of. Every item is one word, so the demanded one-word ordinal answer is well formed.
const COUNT_LISTS: &[(&str, &[&str])] = &[
    ("colours", &["red", "orange", "yellow", "green"]),
    ("fruits", &["apple", "pear", "plum", "fig", "peach"]),
    ("animals", &["cat", "dog", "fox", "owl", "hare"]),
    ("cities", &["Paris", "Rome", "Oslo", "Cairo", "Tokyo"]),
    ("tools", &["hammer", "chisel", "plane", "drill"]),
    ("metals", &["iron", "copper", "zinc", "tin", "lead"]),
    ("trees", &["oak", "pine", "birch", "maple"]),
    (
        "planets",
        &["Mercury", "Venus", "Mars", "Jupiter", "Saturn"],
    ),
    ("rivers", &["Nile", "Rhine", "Volga", "Danube"]),
    (
        "languages",
        &["Latin", "Greek", "Arabic", "Hebrew", "Persian"],
    ),
    ("shapes", &["circle", "square", "triangle", "oval"]),
    ("instruments", &["flute", "violin", "drum", "harp", "piano"]),
];

const COUNT_ASKS: &[&str] = &[
    "Name some {cat}.",
    "List a few {cat} for me.",
    "Which {cat} can you name?",
    "Give me some examples of {cat}.",
    "Tell me a few {cat}.",
];

const COUNT_REPLIES: &[&str] = &[
    "Some {cat} are {list}.",
    "Here are a few {cat}: {list}.",
    "I can name these {cat}: {list}.",
    "The {cat} I know best are {list}.",
];

/// Turn 2 demands a number and nothing else; the answer is the item count of turn 1's list.
const COUNT_DEMANDS: &[&str] = &[
    "Answer with a number only. How many items were in your previous answer?",
    "Reply with digits only. How many things did you just list?",
    "Answer with a number only. How many items did that list have?",
    "Write just the number. How many items were in your last answer?",
    "Answer using a single number. What was the number of items in your previous answer?",
    "Give the number alone. How many items did you name?",
    "Answer with digits only. Count the items in your previous answer.",
    "Reply with just a number. How many items were in the list you gave?",
];

/// `(ordinal word, slot)`: `usize::MAX` means "the last item".
const ORDINAL_SLOTS: &[(&str, usize)] = &[
    ("first", 0),
    ("second", 1),
    ("third", 2),
    ("fourth", 3),
    ("fifth", 4),
    ("last", usize::MAX),
];

const ORDINAL_PREFIXES: &[&str] = &[
    "Answer with one word only.",
    "Reply with exactly one word.",
    "Give a one-word answer.",
    "Answer in a single word.",
    "Keep it to one word.",
];

const ORDINAL_QUESTIONS: &[&str] = &[
    "What was the {ordinal} item in that list?",
    "Which item came {ordinal} in the list you gave?",
    "Name the {ordinal} item in your previous answer.",
    "Which item of that list was {ordinal}?",
];

/// `(country, one-word capital)`: turn 1 answers with a sentence naming the city, turn 2 asks for
/// that one word back.
const CITY_FACTS: &[(&str, &str)] = &[
    ("France", "Paris"),
    ("Italy", "Rome"),
    ("Spain", "Madrid"),
    ("Portugal", "Lisbon"),
    ("Austria", "Vienna"),
    ("Norway", "Oslo"),
    ("Ireland", "Dublin"),
    ("Greece", "Athens"),
    ("Egypt", "Cairo"),
    ("Japan", "Tokyo"),
    ("Korea", "Seoul"),
    ("Peru", "Lima"),
    ("Kenya", "Nairobi"),
    ("Canada", "Ottawa"),
    ("Australia", "Canberra"),
    ("Cuba", "Havana"),
    ("Iran", "Tehran"),
    ("Iraq", "Baghdad"),
    ("Denmark", "Copenhagen"),
    ("Finland", "Helsinki"),
    ("Poland", "Warsaw"),
    ("Chile", "Santiago"),
    ("Sweden", "Stockholm"),
    ("Hungary", "Budapest"),
    ("Morocco", "Rabat"),
];

const CITY_ASKS: &[&str] = &[
    "What is the capital of {country}?",
    "Which city is the capital of {country}?",
    "Name the capital of {country}.",
];

const CITY_REPLIES: &[&str] = &[
    "The capital of {country} is {city}.",
    "The capital city of {country} is {city}.",
    "In {country}, the capital is {city}.",
    "{country} has its capital at {city}.",
];

const CITY_DEMANDS: &[&str] = &[
    "Answer with one word only. Name that city.",
    "Reply with exactly one word. What city was that?",
    "Give a one-word answer. Which city did you just name?",
    "Answer in a single word. Say the city you named.",
    "Use one word only. What was the city?",
    "Respond with just one word. What city did I ask about?",
];

/// `(element, one-word symbol)`.
const SYMBOL_FACTS: &[(&str, &str)] = &[
    ("gold", "Au"),
    ("silver", "Ag"),
    ("iron", "Fe"),
    ("copper", "Cu"),
    ("zinc", "Zn"),
    ("tin", "Sn"),
    ("lead", "Pb"),
    ("mercury", "Hg"),
    ("sodium", "Na"),
    ("potassium", "K"),
    ("calcium", "Ca"),
    ("helium", "He"),
    ("neon", "Ne"),
    ("argon", "Ar"),
    ("silicon", "Si"),
    ("nickel", "Ni"),
    ("cobalt", "Co"),
    ("manganese", "Mn"),
    ("chromium", "Cr"),
    ("platinum", "Pt"),
];

const SYMBOL_ASKS: &[&str] = &[
    "What is the chemical symbol for {element}?",
    "Which symbol stands for {element}?",
];

const SYMBOL_REPLIES: &[&str] = &[
    "The chemical symbol for {element} is {symbol}.",
    "The symbol for the element {element} is {symbol}.",
    "In chemistry, {element} is written {symbol}.",
];

const SYMBOL_DEMANDS: &[&str] = &[
    "Answer with the symbol only. Give that symbol.",
    "Reply with one word only. What symbol was that?",
    "Answer with one word only. Which symbol did you just give?",
    "Give a one-word answer. Say the symbol you named.",
    "Use one word only. What was the symbol?",
];

/// `(description, one-word planet name)`.
const PLANET_FACTS: &[(&str, &str)] = &[
    ("closest to the Sun", "Mercury"),
    ("known for its rings", "Saturn"),
    ("largest in the Solar System", "Jupiter"),
    ("called the red planet", "Mars"),
    ("nearest to Earth", "Venus"),
    ("farthest from the Sun", "Neptune"),
    ("tilted on its side", "Uranus"),
];

const PLANET_ASKS: &[&str] = &[
    "Which planet is {description}?",
    "Name the planet {description}.",
];

const PLANET_REPLIES: &[&str] = &[
    "The planet {description} is {name}.",
    "The planet that is {description} is {name}.",
    "In our Solar System, the planet {description} is {name}.",
];

const PLANET_DEMANDS: &[&str] = &[
    "Answer with one word only. Name that planet.",
    "Reply with exactly one word. Which planet was that?",
    "Give a one-word answer. Say the planet you named.",
    "Use one word only. What was the planet?",
];

/// `(turn-1 question, turn-1 answer, the value the answer states)`. Turn 1 states its result in
/// words and digits; turn 2 asks for an arithmetic transformation of that number.
const NUMBER_FACTS: &[(&str, &str, u64)] = &[
    ("What is ten plus ten?", "Ten plus ten is 20.", 20),
    ("What is seven times three?", "Seven times three is 21.", 21),
    ("What is eight plus five?", "Eight plus five is 13.", 13),
    ("What is nine times four?", "Nine times four is 36.", 36),
    ("What is twelve plus seven?", "Twelve plus seven is 19.", 19),
    ("What is six times six?", "Six times six is 36.", 36),
    (
        "What is fifteen minus four?",
        "Fifteen minus four is 11.",
        11,
    ),
    (
        "What is twenty divided by five?",
        "Twenty divided by five is 4.",
        4,
    ),
    ("What is eleven plus nine?", "Eleven plus nine is 20.", 20),
    (
        "What is thirty minus twelve?",
        "Thirty minus twelve is 18.",
        18,
    ),
    ("What is five times eight?", "Five times eight is 40.", 40),
    ("What is fourteen plus six?", "Fourteen plus six is 20.", 20),
    ("What is forty minus nine?", "Forty minus nine is 31.", 31),
    ("What is three times nine?", "Three times nine is 27.", 27),
    (
        "What is eighteen plus four?",
        "Eighteen plus four is 22.",
        22,
    ),
    ("What is fifty minus eight?", "Fifty minus eight is 42.", 42),
    ("What is seven plus seven?", "Seven plus seven is 14.", 14),
    (
        "What is twenty-five plus five?",
        "Twenty-five plus five is 30.",
        30,
    ),
    (
        "What is sixteen minus seven?",
        "Sixteen minus seven is 9.",
        9,
    ),
    ("What is four times seven?", "Four times seven is 28.", 28),
    (
        "What is thirteen plus eight?",
        "Thirteen plus eight is 21.",
        21,
    ),
    (
        "What is sixty minus fifteen?",
        "Sixty minus fifteen is 45.",
        45,
    ),
    ("What is two times eleven?", "Two times eleven is 22.", 22),
    (
        "What is thirty-three plus nine?",
        "Thirty-three plus nine is 42.",
        42,
    ),
];

/// A numeric derivation over turn 1's stated number: the follow-up answer is computed, not copied.
#[derive(Clone, Copy)]
enum Op {
    Add(u64),
    Sub(u64),
    Mul(u64),
}

/// Turn 2 demands a number and nothing else, then applies `Op` to turn 1's number.
const NUMBER_DEMANDS: &[(&str, Op)] = &[
    (
        "Answer with digits only. What is that number plus one?",
        Op::Add(1),
    ),
    (
        "Reply with a number only. What is that number minus one?",
        Op::Sub(1),
    ),
    (
        "Answer with digits only. What is that number minus two?",
        Op::Sub(2),
    ),
    (
        "Write the number alone. What is that number plus two?",
        Op::Add(2),
    ),
    (
        "Answer using a single number. What is that number times two?",
        Op::Mul(2),
    ),
    (
        "Respond with a number and nothing else. What is that number times three?",
        Op::Mul(3),
    ),
    (
        "Answer with a number only. What is that number plus ten?",
        Op::Add(10),
    ),
    (
        "Give the number alone. What is that number minus three?",
        Op::Sub(3),
    ),
    (
        "Answer with digits only. What is that number plus three?",
        Op::Add(3),
    ),
    (
        "Reply with just a number. What is that number times four?",
        Op::Mul(4),
    ),
    (
        "Answer with a number only. What is that number minus five?",
        Op::Sub(5),
    ),
    (
        "Write just the number. What is that number plus five?",
        Op::Add(5),
    ),
];

/// `(turn-1 question, turn-1 answer, whether the answer is affirmative)`. Turn 1 answers with a
/// sentence whose polarity word is part of it; turn 2 asks yes or no about that answer.
const JUDGEMENTS: &[(&str, &str, bool)] = &[
    ("Is the sky blue?", "Yes, the sky is blue.", true),
    ("Is water wet?", "Yes, water is wet.", true),
    ("Do birds sing?", "Yes, birds sing.", true),
    (
        "Does the moon shine at night?",
        "Yes, the moon shines at night.",
        true,
    ),
    ("Can fish swim?", "Yes, fish can swim.", true),
    ("Do bees make honey?", "Yes, bees make honey.", true),
    ("Does rain fall down?", "Yes, rain falls down.", true),
    ("Is the desert dry?", "Yes, the desert is dry.", true),
    ("Do stars twinkle?", "Yes, stars twinkle.", true),
    ("Is grass green?", "Yes, grass is green.", true),
    ("Do dogs bark?", "Yes, dogs bark.", true),
    ("Is snow cold?", "Yes, snow is cold.", true),
    ("Do plants need light?", "Yes, plants need light.", true),
    (
        "Does ice melt in the sun?",
        "Yes, ice melts in the sun.",
        true,
    ),
    (
        "Is the sun a planet?",
        "No, the sun is not a planet.",
        false,
    ),
    ("Is ice hot?", "No, ice is not hot.", false),
    ("Is a stone alive?", "No, a stone is not alive.", false),
    ("Is fire cold?", "No, fire is not cold.", false),
    ("Is a rock soft?", "No, a rock is not soft.", false),
    ("Is paper heavy?", "No, paper is not heavy.", false),
    (
        "Is the ocean fresh water?",
        "No, the ocean is not fresh water.",
        false,
    ),
    ("Is the sun cold?", "No, the sun is not cold.", false),
    ("Do cats bark?", "No, cats do not bark.", false),
    ("Is a feather heavy?", "No, a feather is not heavy.", false),
    (
        "Does the sun rise in the west?",
        "No, the sun does not rise in the west.",
        false,
    ),
    (
        "Is a whale an insect?",
        "No, a whale is not an insect.",
        false,
    ),
    ("Is the moon a star?", "No, the moon is not a star.", false),
    ("Do penguins fly?", "No, penguins do not fly.", false),
];

/// `(phrasing, asks whether the previous answer was "No")`.
const POLARITY_DEMANDS: &[(&str, bool)] = &[
    ("Answer yes or no. Was your previous answer no?", true),
    ("Reply with yes or no. Was your last answer yes?", false),
    ("Say yes or no. Did you just answer no?", true),
    (
        "Answer with a single word, yes or no. Was your previous answer yes?",
        false,
    ),
    ("Just say yes or no. Did you answer yes before?", false),
    ("Answer yes or no only. Was your last answer no?", true),
    (
        "Reply with one word, yes or no. Was your previous answer no?",
        true,
    ),
    (
        "Answer with one word only, yes or no. Was your last answer yes?",
        false,
    ),
];

/// `(turn-1 question, whether turn 1's bare one-word answer is affirmative)`. Turn 1 answers with
/// the single word `Yes.` or `No.`; turn 2 asks the opposite question, so the demanded one-word
/// answer is the negation and a verbatim copy of turn 1 fails.
const FLIP_FACTS: &[(&str, bool)] = &[
    ("Is the sky blue?", true),
    ("Is the sun a planet?", false),
    ("Is water wet?", true),
    ("Is ice hot?", false),
    ("Do birds sing?", true),
    ("Is a stone alive?", false),
    ("Does the moon shine at night?", true),
    ("Is fire cold?", false),
    ("Can fish swim?", true),
    ("Is a rock soft?", false),
    ("Do bees make honey?", true),
    ("Is paper heavy?", false),
    ("Does rain fall down?", true),
    ("Is the desert dry?", true),
    ("Is a feather heavy?", false),
    ("Do stars twinkle?", true),
    ("Is the ocean fresh water?", false),
    ("Is grass green?", true),
    ("Is the sun cold?", false),
    ("Do cats bark?", false),
];

/// `(object, turn-1 question, attribute kind, one-word value)`. Turn 1 names the object and its
/// attribute in a sentence; turn 2 asks for that one attribute word.
const ATTRIBUTES: &[(&str, &str, &str, &str)] = &[
    ("rose", "What colour is the rose?", "colour", "red"),
    ("bicycle", "What colour is the bicycle?", "colour", "blue"),
    ("door", "What colour is the door?", "colour", "green"),
    ("van", "What colour is the van?", "colour", "white"),
    ("hat", "What colour is the hat?", "colour", "black"),
    ("wall", "What colour is the wall?", "colour", "yellow"),
    ("cup", "What colour is the cup?", "colour", "orange"),
    ("book", "What colour is the book?", "colour", "brown"),
    ("shirt", "What colour is the shirt?", "colour", "pink"),
    ("fence", "What colour is the fence?", "colour", "white"),
    ("boat", "What colour is the boat?", "colour", "blue"),
    ("lamp", "What colour is the lamp?", "colour", "yellow"),
    ("roof", "What colour is the roof?", "colour", "red"),
    ("box", "What colour is the box?", "colour", "brown"),
    ("table", "What shape is the table?", "shape", "round"),
    ("window", "What shape is the window?", "shape", "square"),
    ("coin", "What shape is the coin?", "shape", "round"),
    ("crate", "What material is the crate?", "material", "wooden"),
    ("spoon", "What material is the spoon?", "material", "metal"),
    (
        "bottle",
        "What material is the bottle?",
        "material",
        "glass",
    ),
    ("toy", "What material is the toy?", "material", "plastic"),
    ("elephant", "What size is the elephant?", "size", "large"),
    ("mouse", "What size is the mouse?", "size", "small"),
    ("whale", "What size is the whale?", "size", "large"),
];

const ATTRIBUTE_REPLIES: &[&str] = &[
    "The {object} is {value}.",
    "The {object} looks {value}.",
    "I would say the {object} is {value}.",
];

const ATTRIBUTE_DEMANDS: &[&str] = &[
    "Answer with one word only. What {kind} was the {object} I named?",
    "Reply with exactly one word. What was the {kind} of the {object}?",
    "Give a one-word answer. Which {kind} did the {object} have?",
    "Answer in a single word. Say the {kind} of the {object}.",
    "Use one word only. What {kind} did I ask about?",
    "Respond with just one word. What {kind} was it?",
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
        eprintln!("demand-store-attr: {error}");
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

    let (specs, family_report) = select_specs(args.seed)?;
    let documents = build_documents(&specs, &tokenizer)?;
    let mut stream: Vec<u16> = Vec::new();
    let mut mask: Vec<u8> = Vec::new();
    for document in &documents {
        stream.extend_from_slice(&document.tokens);
        mask.extend_from_slice(&document.mask);
    }

    println!(
        "emitted: {} two-turn documents, {} tokens, {} response tokens, {} response runs",
        documents.len(),
        stream.len(),
        mask.iter().filter(|&&value| value == 1).count(),
        2 * documents.len()
    );
    for (family, (enumerated, emitted)) in &family_report {
        println!("  family {family}: {emitted} documents from {enumerated} candidates");
    }

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
                    "usage: demand-store-attr <output-dir> [--tokenizer <tokenizer.json>] \
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
    "usage: demand-store-attr <output-dir> [--tokenizer <tokenizer.json>] [--seed N] \
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

/// `a, b, c and d` — the list rendering both the count and the ordinal family read.
fn join_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [only] => (*only).to_string(),
        [rest @ .., last] => format!("{} and {}", rest.join(", "), last),
    }
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

fn count_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(category, items) in COUNT_LISTS {
        let list = join_list(items);
        let count = items.len();
        for ask in COUNT_ASKS {
            let question1 = fill(ask, "{cat}", category);
            for reply in COUNT_REPLIES {
                let answer1 = fill(&fill(reply, "{cat}", category), "{list}", &list);
                for demand in COUNT_DEMANDS {
                    specs.push(Spec {
                        family: "count",
                        question1: question1.clone(),
                        answer1: answer1.clone(),
                        question2: (*demand).to_string(),
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
    for &(category, items) in COUNT_LISTS {
        let list = join_list(items);
        for ask in COUNT_ASKS {
            let question1 = fill(ask, "{cat}", category);
            for reply in COUNT_REPLIES {
                let answer1 = fill(&fill(reply, "{cat}", category), "{list}", &list);
                for &(ordinal, slot) in ORDINAL_SLOTS {
                    let index = if slot == usize::MAX {
                        items.len() - 1
                    } else {
                        slot
                    };
                    if index >= items.len() {
                        continue;
                    }
                    let item = items[index];
                    for prefix in ORDINAL_PREFIXES {
                        for question in ORDINAL_QUESTIONS {
                            specs.push(Spec {
                                family: "ordinal",
                                question1: question1.clone(),
                                answer1: answer1.clone(),
                                question2: format!(
                                    "{prefix} {}",
                                    fill(question, "{ordinal}", ordinal)
                                ),
                                answer2: format!("{item}."),
                            });
                        }
                    }
                }
            }
        }
    }
    specs
}

fn name_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(country, city) in CITY_FACTS {
        for ask in CITY_ASKS {
            let question1 = fill(ask, "{country}", country);
            for reply in CITY_REPLIES {
                let answer1 = fill(&fill(reply, "{country}", country), "{city}", city);
                for demand in CITY_DEMANDS {
                    specs.push(Spec {
                        family: "name",
                        question1: question1.clone(),
                        answer1: answer1.clone(),
                        question2: (*demand).to_string(),
                        answer2: format!("{city}."),
                    });
                }
            }
        }
    }
    for &(element, symbol) in SYMBOL_FACTS {
        for ask in SYMBOL_ASKS {
            let question1 = fill(ask, "{element}", element);
            for reply in SYMBOL_REPLIES {
                let answer1 = fill(&fill(reply, "{element}", element), "{symbol}", symbol);
                for demand in SYMBOL_DEMANDS {
                    specs.push(Spec {
                        family: "name",
                        question1: question1.clone(),
                        answer1: answer1.clone(),
                        question2: (*demand).to_string(),
                        answer2: format!("{symbol}."),
                    });
                }
            }
        }
    }
    for &(description, name) in PLANET_FACTS {
        for ask in PLANET_ASKS {
            let question1 = fill(ask, "{description}", description);
            for reply in PLANET_REPLIES {
                let answer1 = fill(&fill(reply, "{description}", description), "{name}", name);
                for demand in PLANET_DEMANDS {
                    specs.push(Spec {
                        family: "name",
                        question1: question1.clone(),
                        answer1: answer1.clone(),
                        question2: (*demand).to_string(),
                        answer2: format!("{name}."),
                    });
                }
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
    for &(question1, answer1, value) in NUMBER_FACTS {
        for (demand, op) in NUMBER_DEMANDS {
            let Some(derived) = derive(value, *op) else {
                continue;
            };
            if derived == value {
                continue;
            }
            specs.push(Spec {
                family: "number",
                question1: question1.to_string(),
                answer1: answer1.to_string(),
                question2: (*demand).to_string(),
                answer2: format!("{derived}."),
            });
        }
    }
    specs
}

fn polarity_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(question1, answer1, affirmative) in JUDGEMENTS {
        let was_no = !affirmative;
        for &(demand, asks_no) in POLARITY_DEMANDS {
            let answer2 = if was_no == asks_no { "Yes." } else { "No." };
            specs.push(Spec {
                family: "polarity",
                question1: question1.to_string(),
                answer1: answer1.to_string(),
                question2: demand.to_string(),
                answer2: answer2.to_string(),
            });
        }
    }
    specs
}

fn flip_word_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(question1, affirmative) in FLIP_FACTS {
        let answer1 = if affirmative { "Yes." } else { "No." };
        let was_no = !affirmative;
        for &(demand, asks_no) in POLARITY_DEMANDS {
            // Only the phrasings that ask "was your previous answer no?" can be answered by the
            // negation of a bare `Yes.`/`No.`; the others would demand a copy of it.
            if !asks_no {
                continue;
            }
            let answer2 = if was_no == asks_no { "Yes." } else { "No." };
            specs.push(Spec {
                family: "flip_word",
                question1: question1.to_string(),
                answer1: answer1.to_string(),
                question2: demand.to_string(),
                answer2: answer2.to_string(),
            });
        }
    }
    specs
}

fn attribute_specs() -> Vec<Spec> {
    let mut specs = Vec::new();
    for &(object, question, kind, value) in ATTRIBUTES {
        for reply in ATTRIBUTE_REPLIES {
            let answer1 = fill(&fill(reply, "{object}", object), "{value}", value);
            for demand in ATTRIBUTE_DEMANDS {
                specs.push(Spec {
                    family: "attribute",
                    question1: question.to_string(),
                    answer1: answer1.clone(),
                    question2: fill(&fill(demand, "{kind}", kind), "{object}", object),
                    answer2: format!("{}.", capitalize(value)),
                });
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
        ("flip_word", flip_word_specs()),
        ("attribute", attribute_specs()),
    ]
}

/// Shuffle every family with the seed, truncate it to its quota, then shuffle the union: the
/// document order and membership are fixed by the seed alone, and no candidate can be emitted
/// twice.
fn select_specs(seed: u64) -> Result<(Vec<Spec>, BTreeMap<&'static str, (usize, usize)>), String> {
    let mut selected: Vec<Spec> = Vec::new();
    let mut report: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for (family, mut specs) in candidate_families() {
        let enumerated = specs.len();
        if enumerated == 0 {
            return Err(format!("family {family} enumerated no candidates"));
        }
        let quota = quota_for(family)?;
        shuffle(&mut specs, family_seed(seed, family));
        specs.truncate(quota);
        report.insert(family, (enumerated, specs.len()));
        selected.extend(specs);
    }
    let total: usize = report.values().map(|entry| entry.1).sum();
    if total < 2000 {
        return Err(format!(
            "only {total} documents were selected; the store requires at least 2000"
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
         ({:.6}), of the whole prefix before turn 2 {} ({:.6})",
        stats.span_in_answer1,
        stats.span_in_answer1 as f64 / documents as f64,
        stats.span_in_prefix,
        stats.span_in_prefix as f64 / documents as f64
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
        "generator": "crates/uor-r4-training/examples/demand-store-attr.rs",
        "mask_bytes": mask.len(),
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating \
                      <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, \
                      and an unmasked document-terminal <|eos|>.",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "mask_sha256": uor_r4_training::sha256_bytes(mask),
        "response_tokens": mask.iter().filter(|&&value| value == 1).count(),
        "schema": "uor-r4-demand-store-attr/v1",
        "seed": format!("0x{:016x}", args.seed),
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
