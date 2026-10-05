//! M-world v2 (Lab 1, A1): the retrieval instrument. Every episode is answerable
//! only by reading the context, and the evaluator measures retrieval by the
//! token distance between an assertion and its query.
//!
//! v1 (`milestone_world`) is R1's sealed instrument and is untouched: its
//! rendering, `judge()` and `digest()` are pinned by the tests below. v2 reuses
//! v1's types and generators where they are unchanged and adds its own judge
//! ([`judge_v2`]) for the four oracle defects of #1516, which v1 keeps.
//!
//! What v2 adds:
//!
//! - **Open value pools.** Names, code words, hometown stems and numbers are
//!   drawn from a deterministic syllable generator (onset x vowel x coda, 2-3
//!   syllables) instead of a table of about ten values per relation, so a value
//!   cannot be memorized. The Train/Development split is a keyed hash of the
//!   lowercased string ([`split_of`]): a string belongs to exactly one split by
//!   construction, and each split's universe holds millions of strings. v1's
//!   closed relations remain as a minority, tagged [`Pool::Closed`].
//! - **MQAR episodes.** The user gives N key/value pairs in one turn, the
//!   assistant acknowledges, responsive filler turns follow until the distance
//!   reaches D, and the user asks for one key. The distance is measured in the
//!   caller's real tokens through a closure ([`Meter`]) and recorded per item.
//!   Every episode of every kind fits [`CONTEXT`] tokens.
//! - **Copy episodes:** "Repeat exactly: w1 w2 ..." judged by exact word
//!   sequence.
//!
//! Distance. `distance` is the number of tokens strictly between the last
//! token of the assertion turn and the first token of the query text (the
//! acknowledgment turn, every filler turn and the query's role markers), so it
//! does not depend on which of the N pairs is asked. Measuring from the queried
//! pair itself would force the queried pair to be the last one at D=16, and the
//! answer "the most recent value" would pass; the item also records
//! `pair_distance`, which adds the tokens of the assertion that follow the
//! queried pair's value.
//!
//! Revision 2.1 (the council's A1 amendments, made before any treatment run):
//!
//! - **Cross cells.** An episode can be drawn under a [`Cell`]: the phrasing
//!   of its user turns from one split and its values from the same or the
//!   other. `dev_phrasing x dev_value` is the development split, drawn exactly
//!   as before (`conversation(rng, split)` is `conversation_in` under
//!   [`Cell::same`]) and the only cell the A1 gate is decided on;
//!   `train_phrasing x dev_value` is the pure-retrieval cell (a trained
//!   phrasing around a value never seen). [`CellScores`] reports every cell.
//! - **Untrained rule baselines** ([`Rule`]): R-recency (the latest open value
//!   in the history) and R-nlet (the continuation of the latest earlier
//!   occurrence of the query's last two words), judged by [`judge_v2`] on the
//!   same items. The instrument freezes only if both stay below
//!   [`FREEZE_LIMIT`] on every gated cell ([`freeze_report`]).
//! - **The generator revision that condition demanded.** Reading the
//!   generator against the two rules found two leaks: a relation episode
//!   stated one relation only, so the queried value was always the latest open
//!   value (R-recency), and four MQAR query phrasings ended in "{k} is", the
//!   two words that precede the value in the assertion (R-nlet). No MQAR query
//!   phrasing ends in "{k} is" any more, and a relation episode states a second
//!   relation (see the next item).
//!   The rules were not run when this was written (the machine was on hold);
//!   `m-world baselines` measures them on the revised world.
//! - **Balanced relation queries (#1541, before any run).** In the first form
//!   of revision 2.1 a relation episode stated a second relation after the
//!   queried one (with probability [`DISTRACTOR`]) and never asked it, so the
//!   answer was "the value stated before the last statement", found without
//!   reading the query. Every relation episode now states a companion relation
//!   of the queried relation's pool (open with open, closed with closed), and
//!   the query asks either of the two on a fair coin drawn from the episode's
//!   seeded stream, whatever order they were stated in. [`DISTRACTOR`] is now
//!   only the order: the share of episodes with an update that state the
//!   companion after it (otherwise the update comes last). Each tested
//!   query-blind rule over the stated values (the latest, the first, the k-th)
//!   is right on at most about half of the items; some score less, since the
//!   first raw value can be stale after an update. The companion is always
//!   stated because a share `s` of one-relation episodes, which every such
//!   rule answers, lets the best of them score `s + (1 - s) / 2` however fair
//!   the coin: 0.625 at the former `s = 1/4`, over the 0.6 [`FREEZE_LIMIT`].
//!   The stream this generator draws is pinned by a digest in the tests;
//!   [`MWorld2::digest`] hashes tables and constants only and did not move.
//! - **Recorded facts.** A [`Tag`] carries the phrasing template, the gold
//!   value and the open values a turn states, for the rules, the per-template
//!   leak report and the teacher-forced answer-span scores.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::milestone_world::{
    articles, capitalize, closer, contains_phrase, fill, instruction, number, opener, pick,
    responsive, set_train_paraphrases, slots, strings, words, Category, Check, MWorld, Phrasings,
    Split, TrainParaphrases, Turn, ABSENT_ACCEPT, ABSENT_REPLIES, ACK_WORDS, RELATIONS,
};
use crate::stack_tracking::Rng;
use crate::{invalid, Result};

/// The model context, in tokens. Every episode fits.
pub const CONTEXT: usize = 256;

/// The MQAR distances (tokens between assertion and query) and pair counts.
pub const DISTANCES: [usize; 3] = [16, 64, 200];
pub const PAIR_COUNTS: [usize; 3] = [2, 4, 8];

/// How far past D an achieved MQAR distance may fall and still count as D.
pub fn tolerance(distance: usize) -> usize {
    (distance / 8).max(8)
}

/// The fraction of relation episodes that use v1's closed relations.
pub const CLOSED_SHARE: f64 = 0.2;

/// The instrument's revision, part of [`MWorld2::digest`]. `2.0` is the world
/// as first written (c5b84173); `2.1` adds the council's A1 amendments, with
/// the balanced relation queries of #1541 (made before any treatment run).
pub const REVISION: &str = "2.1";

/// The share (numerator, denominator) of relation episodes that state the
/// companion relation after the queried relation's last statement. In an
/// episode that updates the queried relation, the update comes last otherwise
/// (the companion falls between the assertion and the update); an episode with
/// no update states the companion last either way. Every relation episode
/// states a companion: which of the two relations is asked is a separate fair
/// coin, so this share never tells the query.
pub const DISTRACTOR: (usize, usize) = (3, 4);

/// The (phrasing split x value split) cell an item is drawn from. The four
/// cells separate a failure to read a new phrasing from a failure to copy a
/// new value. `dev_phrasing x dev_value` is the development split and the only
/// cell the A1 gate is decided on ([`Cell::GATED`]); `train_phrasing x
/// dev_value` is the pure-retrieval cell ([`Cell::PURE_RETRIEVAL`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Cell {
    /// The split of the user-turn templates (and the filler turns).
    pub phrasing: Split,
    /// The split of the names, numbers, code words, towns and closed values.
    pub value: Split,
}

impl Cell {
    /// The four cells, in a fixed order.
    pub const ALL: [Cell; 4] = [
        Cell::new(Split::Train, Split::Train),
        Cell::new(Split::Train, Split::Development),
        Cell::new(Split::Development, Split::Train),
        Cell::new(Split::Development, Split::Development),
    ];

    /// The development split (development phrasing, development values): the
    /// A1 gate is computed here, as before.
    pub const GATED: Cell = Cell::new(Split::Development, Split::Development);

    /// Trained phrasing, unseen values: retrieval with the phrasing held fixed.
    pub const PURE_RETRIEVAL: Cell = Cell::new(Split::Train, Split::Development);

    pub const fn new(phrasing: Split, value: Split) -> Self {
        Self { phrasing, value }
    }

    /// The cell of a plain split: both from `split`.
    pub const fn same(split: Split) -> Self {
        Self::new(split, split)
    }

    /// The cell's name in reports.
    pub const fn key(self) -> &'static str {
        match (self.phrasing, self.value) {
            (Split::Train, Split::Train) => "train_phrasing_train_value",
            (Split::Train, Split::Development) => "train_phrasing_dev_value",
            (Split::Development, Split::Train) => "dev_phrasing_train_value",
            (Split::Development, Split::Development) => "dev_phrasing_dev_value",
        }
    }
}

// ---------------------------------------------------------------------------
// Judging: v1's checks plus the #1516 fixes.

/// One check of the v2 oracle. Every v1 check that #1516 does not touch runs
/// unchanged through [`Check2::V1`]; the four defective ones and the copy check
/// are v2 variants. A reply passes when every check of its turn holds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Check2 {
    /// A v1 check, judged exactly as v1 judges it. Never `Larger`,
    /// `LastLetter` or `Numbers`: [`Check2::from_v1`] maps those.
    V1(Check),
    /// [`Check::Larger`] without the echo and negation-window defects.
    Larger { answer: u32, other: u32 },
    /// [`Check::LastLetter`], case-insensitive.
    LastLetter(char),
    /// [`Check::Numbers`] without the pronoun "one".
    Numbers(Vec<u32>),
    /// The reply's words, lowercased and without punctuation, are exactly
    /// these words in this order.
    CopyExact(Vec<String>),
}

impl Check2 {
    /// v1's check under v2 semantics.
    pub fn from_v1(check: &Check) -> Self {
        match check {
            Check::Larger { answer, other } => Self::Larger {
                answer: *answer,
                other: *other,
            },
            Check::LastLetter(letter) => Self::LastLetter(*letter),
            Check::Numbers(values) => Self::Numbers(values.clone()),
            other => Self::V1(other.clone()),
        }
    }
}

/// The v2 oracle: whether `reply` answers `user` under `checks`.
pub fn judge_v2(checks: &[Check2], user: &str, reply: &str) -> bool {
    checks.iter().all(|check| match check {
        Check2::V1(check) => {
            crate::milestone_world::judge(std::slice::from_ref(check), user, reply)
        }
        Check2::Larger { answer, other } => larger(reply, user, *answer, *other),
        Check2::LastLetter(letter) => last_letter(reply) == Some(letter.to_ascii_uppercase()),
        Check2::Numbers(values) => {
            let present = numerals(reply);
            values.iter().all(|v| present.contains(v))
        }
        Check2::CopyExact(expected) => words(reply) == *expected,
    })
}

const MORE: [&str; 8] = [
    "bigger", "larger", "greater", "higher", "biggest", "largest", "greatest", "highest",
];
const LESS: [&str; 4] = ["smaller", "less", "lower", "fewer"];

/// Clauses of a reply: words between punctuation and the conjunctions that
/// begin a new statement. A comparative applies within its clause only.
fn clauses(text: &str) -> Vec<Vec<String>> {
    text.split(|c: char| matches!(c, ',' | ';' | ':' | '.' | '!' | '?' | '(' | ')' | '\n'))
        .flat_map(|piece| {
            let mut clauses = vec![Vec::new()];
            for word in words(piece) {
                if matches!(word.as_str(), "and" | "but" | "while" | "whereas") {
                    clauses.push(Vec::new());
                } else if let Some(last) = clauses.last_mut() {
                    last.push(word);
                }
            }
            clauses
        })
        .filter(|clause| !clause.is_empty())
        .collect()
}

pub(crate) fn contains_slice(haystack: &[String], needle: &[String]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack.windows(needle.len()).any(|w| w == needle)
}

/// A number word or digits; the pronoun "one" is not a value here.
fn operand(word: &str) -> Option<u32> {
    if word == "one" {
        None
    } else {
        number(word)
    }
}

/// Whether some clause states `value` as the larger number: "{value} is
/// bigger" (no "than" before the value, no smaller/less word after it) or "the
/// bigger one is {value}". Evidence that is a phrase of the user's own turn is
/// an echo of the prompt, not a statement.
fn states_larger(clause: &[String], value: u32, user: &[String]) -> bool {
    for (i, word) in clause.iter().enumerate() {
        if operand(word) != Some(value) || (i > 0 && clause[i - 1] == "than") {
            continue;
        }
        let after = &clause[i + 1..(i + 4).min(clause.len())];
        let comparative = after
            .iter()
            .take_while(|w| w.as_str() != "than")
            .position(|w| MORE.contains(&w.as_str()));
        if let Some(offset) = comparative {
            let negated = after.iter().any(|w| LESS.contains(&w.as_str()));
            if !negated && !contains_slice(user, &clause[i..=i + 1 + offset]) {
                return true;
            }
        }
        if i > 0 && clause[i - 1] == "is" {
            let start = i.saturating_sub(4);
            let before = &clause[start..i - 1];
            if let Some(offset) = before.iter().position(|w| MORE.contains(&w.as_str())) {
                let negated = before[offset..].iter().any(|w| LESS.contains(&w.as_str()));
                if !negated && !contains_slice(user, &clause[start + offset..=i]) {
                    return true;
                }
            }
        }
    }
    false
}

/// [`Check2::Larger`]. A reply that contains the user's whole turn, or whose
/// only evidence for the answer is a phrase of that turn ("Out of 3 and 7,
/// which is greater?"), does not pass. A comparative negated only by a later
/// clause ("The bigger number is 7, the smaller is 3") is not negated. A reply
/// that also calls the other number larger contradicts itself and fails.
fn larger(reply: &str, user: &str, answer: u32, other: u32) -> bool {
    let user_words = words(user);
    let reply_words = words(reply);
    if !user_words.is_empty() && contains_phrase(&reply_words, &user_words.join(" ")) {
        return false;
    }
    let clauses = clauses(reply);
    let states = |value| clauses.iter().any(|c| states_larger(c, value, &user_words));
    if states(other) {
        return false;
    }
    if states(answer) {
        return true;
    }
    let named: BTreeSet<u32> = reply_words.iter().filter_map(|w| operand(w)).collect();
    named.contains(&answer)
        && !named.contains(&other)
        && !reply_words.iter().any(|w| LESS.contains(&w.as_str()))
}

/// [`Check2::LastLetter`]'s letter, uppercased: the last standalone
/// single-letter word that does not begin a sentence, or the reply's only
/// word. A lowercase "a" or "i" is the article or the pronoun unless it
/// directly follows the word "letter" or is the whole reply.
fn last_letter(reply: &str) -> Option<char> {
    let mut sentence_start = true;
    let mut found = None;
    let mut count = 0usize;
    let mut previous = String::new();
    for token in reply.split_whitespace() {
        let word: String = token
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '\'')
            .collect();
        if !word.is_empty() {
            count += 1;
            let mut chars = word.chars();
            if let (Some(c), None) = (chars.next(), chars.next()) {
                let ambiguous = matches!(c, 'a' | 'i');
                let counts = c.is_uppercase() || !ambiguous || previous == "letter" || count == 1;
                if c.is_alphabetic() && counts && (!sentence_start || count == 1) {
                    found = Some((c.to_ascii_uppercase(), sentence_start));
                }
            }
            sentence_start = false;
            previous = word.to_lowercase();
        }
        if token.ends_with(['.', '!', '?']) {
            sentence_start = true;
        }
    }
    match found {
        Some((c, true)) if count == 1 => Some(c),
        Some((_, true)) => None,
        Some((c, false)) => Some(c),
        None => None,
    }
}

/// The numbers a reply states, as digits or number words. "one" counts only
/// beside another numeral in the same run of words ("one, two, three"), never
/// as a pronoun ("which one", "here is one: 2, 3"). A colon, semicolon,
/// parenthesis, line break, "!" or "?" ends a run.
fn numerals(reply: &str) -> BTreeSet<u32> {
    let mut present = BTreeSet::new();
    for run in reply.split(|c: char| matches!(c, ':' | ';' | '(' | ')' | '\n' | '!' | '?')) {
        let run = words(run);
        let numeral = |j: Option<usize>| {
            j.and_then(|j| run.get(j))
                .is_some_and(|w| w != "one" && number(w).is_some())
        };
        for (i, word) in run.iter().enumerate() {
            if word == "one" {
                if numeral(i.checked_sub(1)) || numeral(Some(i + 1)) {
                    present.insert(1);
                }
            } else if let Some(n) = number(word) {
                present.insert(n);
            }
        }
    }
    present
}

// ---------------------------------------------------------------------------
// Open value pools.

const SPLIT_KEY: &str = "m-world-v2/value-split/1";

/// One string in this many belongs to Development.
const DEVELOPMENT_ONE_IN: u64 = 4;

const ONSETS: [&str; 26] = [
    "b", "br", "ch", "d", "dr", "f", "g", "gr", "h", "j", "k", "kl", "l", "m", "n", "p", "pl", "r",
    "s", "sh", "st", "t", "tr", "v", "w", "z",
];
const VOWELS: [&str; 8] = ["a", "e", "i", "o", "u", "ai", "ou", "ee"];
const CODAS: [&str; 12] = ["", "n", "m", "r", "l", "s", "k", "t", "d", "nd", "rk", "st"];
const TOWN_SUFFIXES: [&str; 10] = [
    "ville", "burg", "ford", "ton", "field", "mouth", "haven", "stead", "port", "wood",
];

/// Keyed 64-bit hash of the lowercased text: FNV-1a over the key and the text,
/// then a SplitMix64 finalizer.
fn keyed_hash(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let lower = text.to_lowercase();
    for byte in SPLIT_KEY.bytes().chain([0u8]).chain(lower.bytes()) {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^ (h >> 31)
}

/// The split a generated string belongs to, by its keyed hash (case is
/// ignored). A string is never in both splits.
pub fn split_of(text: &str) -> Split {
    if keyed_hash(text) % DEVELOPMENT_ONE_IN == 0 {
        Split::Development
    } else {
        Split::Train
    }
}

/// Words that appear in any fixed text of v1 or v2 (templates, replies,
/// acknowledgments, facts, closed values). A generated value is never one of
/// them, so a value cannot be matched by a template or filler reply.
fn reserved() -> &'static BTreeSet<String> {
    static RESERVED: OnceLock<BTreeSet<String>> = OnceLock::new();
    RESERVED.get_or_init(|| {
        let mut set = BTreeSet::new();
        for split in [Split::Train, Split::Development] {
            let mut rng = Rng::new(0x5EED_0002);
            for _ in 0..6000 {
                for turn in MWorld::conversation(&mut rng, split).turns {
                    set.extend(words(&turn.user));
                    set.extend(words(&turn.reply));
                }
            }
        }
        for text in fixed_texts() {
            set.extend(words(text));
        }
        set
    })
}

/// Every user phrasing template of `split` in v1 and v2.
fn split_templates(split: Split) -> BTreeSet<&'static str> {
    let side = |p: &Phrasings| match split {
        Split::Train => p.train,
        Split::Development => p.development,
    };
    let mut all: BTreeSet<&'static str> = MWorld::templates(split).into_iter().collect();
    for p in [&MQAR_LEAD, &MQAR_QUERY, &COPY] {
        all.extend(side(p));
    }
    for rel in relations() {
        for p in [rel.assert, rel.update, rel.query] {
            all.extend(side(p));
        }
    }
    all
}

/// Install training paraphrases for this process: `(source_template, text)`
/// pairs, each source a training template of v1 or v2 and each text keeping
/// exactly its source's slots. A text is drawn instead of its source in
/// `share_permille` of that template's training picks; development picks are
/// untouched. The reserved words are fixed first, so generated values are
/// the same as without paraphrases. A text equal to any template is
/// rejected. Returns the number of templates with wordings. Fails if a table
/// is already installed.
pub fn install_train_paraphrases(
    pairs: &[(String, String)],
    share_permille: usize,
) -> Result<usize> {
    reserved();
    let paraphrases = train_paraphrases(pairs, share_permille)?;
    let templates = paraphrases.by_template.len();
    set_train_paraphrases(paraphrases)?;
    Ok(templates)
}

/// The validated table [`install_train_paraphrases`] installs.
fn train_paraphrases(
    pairs: &[(String, String)],
    share_permille: usize,
) -> Result<TrainParaphrases> {
    if share_permille > 1000 {
        return Err(invalid("paraphrase share must be at most 1000 per mille"));
    }
    let train = split_templates(Split::Train);
    let development = split_templates(Split::Development);
    let mut by_template: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
    for (source, text) in pairs {
        let source: &'static str = train.get(source.as_str()).copied().ok_or_else(|| {
            invalid(format!(
                "paraphrase source is no training template: {source}"
            ))
        })?;
        if slots(source) != slots(text) {
            return Err(invalid(format!(
                "paraphrase changes the slots of {source:?}: {text:?}"
            )));
        }
        if train.contains(text.as_str()) || development.contains(text.as_str()) {
            return Err(invalid(format!("paraphrase is a world template: {text:?}")));
        }
        let wordings = by_template.entry(source).or_default();
        if !wordings.contains(&text.as_str()) {
            wordings.push(Box::leak(text.clone().into_boxed_str()));
        }
    }
    Ok(TrainParaphrases {
        share_permille,
        by_template,
    })
}

/// Every fixed string of the v2 tables, for the reserved words and the digest.
fn fixed_texts() -> Vec<&'static str> {
    let mut all: Vec<&'static str> = Vec::new();
    all.extend(MQAR_ACKS);
    all.extend(MQAR_REPLIES);
    all.extend(MQAR_LEAD.train);
    all.extend(MQAR_LEAD.development);
    all.extend(MQAR_QUERY.train);
    all.extend(MQAR_QUERY.development);
    all.extend(COPY.train);
    all.extend(COPY.development);
    all.extend(UPDATE_ACKS);
    for rel in relations() {
        for p in [rel.assert, rel.update, rel.query] {
            all.extend(p.train);
            all.extend(p.development);
        }
        all.extend(rel.acks);
        all.extend(rel.answers);
    }
    all.extend(TOWN_SUFFIXES);
    all
}

fn syllable(rng: &mut Rng) -> String {
    format!(
        "{}{}{}",
        pick(rng, &ONSETS),
        pick(rng, &VOWELS),
        pick(rng, &CODAS)
    )
}

/// A lowercase word of `min..=max` syllables that belongs to `split` and is
/// not a reserved word.
pub fn word(rng: &mut Rng, split: Split, min: usize, max: usize) -> Result<String> {
    for _ in 0..100_000 {
        let count = min + rng.below(max - min + 1);
        let candidate: String = (0..count).map(|_| syllable(rng)).collect();
        if split_of(&candidate) == split && !reserved().contains(&candidate) {
            return Ok(candidate);
        }
    }
    Err(invalid(
        "the syllable generator found no word for the split",
    ))
}

/// A capitalized 2-3 syllable name of `split`.
pub fn name(rng: &mut Rng, split: Split) -> Result<String> {
    let mut name = word(rng, split, 2, 3)?;
    capitalize(&mut name);
    Ok(name)
}

/// A number of 2 to 4 digits, as text, that belongs to `split`.
pub fn digits(rng: &mut Rng, split: Split) -> Result<String> {
    for _ in 0..100_000 {
        let length = 2 + rng.below(3) as u32;
        let low = 10usize.pow(length - 1);
        let high = 10usize.pow(length);
        let value = (low + rng.below(high - low)).to_string();
        if split_of(&value) == split {
            return Ok(value);
        }
    }
    Err(invalid("the number generator found no value for the split"))
}

/// A hometown: a syllable stem of `split` and a suffix.
fn town(rng: &mut Rng, split: Split) -> Result<String> {
    let mut stem = word(rng, split, 1, 2)?;
    stem.push_str(pick(rng, &TOWN_SUFFIXES));
    capitalize(&mut stem);
    Ok(stem)
}

/// How many distinct two-syllable strings belong to `split` (reserved words
/// excluded), among those whose first syllable is one of the first `first`
/// syllables of the inventory. The whole inventory has 2,496 syllables, so
/// `first` = 2,496 counts the entire two-syllable universe.
pub fn two_syllable_count(split: Split, first: usize) -> usize {
    let inventory: Vec<String> = ONSETS
        .iter()
        .flat_map(|o| {
            VOWELS
                .iter()
                .flat_map(move |v| CODAS.iter().map(move |c| format!("{o}{v}{c}")))
        })
        .collect();
    let reserved = reserved();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for head in inventory.iter().take(first) {
        for tail in &inventory {
            let candidate = format!("{head}{tail}");
            if split_of(&candidate) == split && !reserved.contains(&candidate) {
                seen.insert(candidate);
            }
        }
    }
    seen.len()
}

/// Every word of a fixed text of v1 or v2 (see `reserved`): no generated value
/// is one of them, and a sealed probe must not use one as a value.
pub fn reserved_words() -> &'static BTreeSet<String> {
    reserved()
}

/// Whether the syllable generator can spell `text` (lowercase ASCII) in one to
/// `max` syllables of onset x vowel x coda.
pub fn is_syllable_word(text: &str, max: usize) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() || !text.is_ascii() {
        return false;
    }
    // fewest[i]: the fewest syllables that spell text[..i].
    let mut fewest = vec![usize::MAX; bytes.len() + 1];
    fewest[0] = 0;
    for start in 0..bytes.len() {
        if fewest[start] == usize::MAX {
            continue;
        }
        for onset in ONSETS {
            let Some(after_onset) = text[start..].strip_prefix(onset) else {
                continue;
            };
            for vowel in VOWELS {
                let Some(after_vowel) = after_onset.strip_prefix(vowel) else {
                    continue;
                };
                for coda in CODAS {
                    if after_vowel.starts_with(coda) {
                        let end = bytes.len() - after_vowel.len() + coda.len();
                        fewest[end] = fewest[end].min(fewest[start] + 1);
                    }
                }
            }
        }
    }
    fewest[bytes.len()] <= max
}

/// Whether one of the world's value generators can produce `text`, compared
/// lowercased: a word or name of one to three syllables, a number of two to
/// four digits, or a town of a one or two syllable stem and a suffix.
pub fn in_generated_universe(text: &str) -> bool {
    let lower = text.to_lowercase();
    let is_number = (2..=4).contains(&lower.len())
        && lower.bytes().all(|b| b.is_ascii_digit())
        && !lower.starts_with('0');
    is_number
        || is_syllable_word(&lower, 3)
        || TOWN_SUFFIXES.iter().any(|suffix| {
            lower
                .strip_suffix(suffix)
                .is_some_and(|stem| is_syllable_word(stem, 2))
        })
}

// ---------------------------------------------------------------------------
// Relations: v1's closed ones and the open ones.

/// Whether a relation's values come from an open pool or a small closed one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Pool {
    Open,
    Closed,
}

enum Values {
    UserName,
    PetName,
    FriendName,
    Hometown,
    LuckyNumber,
    CodeWord,
    Closed(&'static [&'static str], &'static [&'static str]),
}

struct Rel {
    name: &'static str,
    values: Values,
    assert: &'static Phrasings,
    update: &'static Phrasings,
    query: &'static Phrasings,
    acks: &'static [&'static str],
    answers: &'static [&'static str],
}

impl Rel {
    fn pool(&self) -> Pool {
        match self.values {
            Values::Closed(..) => Pool::Closed,
            _ => Pool::Open,
        }
    }

    fn draw(&self, rng: &mut Rng, split: Split) -> Result<String> {
        match self.values {
            Values::UserName | Values::PetName | Values::FriendName => name(rng, split),
            Values::Hometown => town(rng, split),
            Values::LuckyNumber => digits(rng, split),
            Values::CodeWord => word(rng, split, 2, 3),
            Values::Closed(train, development) => Ok((*pick(
                rng,
                match split {
                    Split::Train => train,
                    Split::Development => development,
                },
            ))
            .to_owned()),
        }
    }
}

static FRIEND_ASSERT: Phrasings = Phrasings {
    train: &[
        "My friend is named {v}.",
        "I have a friend called {v}.",
        "My best friend is {v}.",
        "My friend's name is {v}.",
    ],
    development: &["A friend of mine is called {v}.", "My buddy is {v}."],
};
static FRIEND_UPDATE: Phrasings = Phrasings {
    train: &[
        "Actually, my friend is named {v}.",
        "Sorry, my best friend is {v}.",
    ],
    development: &["Correction: my friend is {v}."],
};
static FRIEND_QUERY: Phrasings = Phrasings {
    train: &[
        "What is my friend's name?",
        "Who is my friend?",
        "What's my friend called?",
    ],
    development: &[
        "Remind me who my friend is.",
        "What did I say my friend is named?",
    ],
};
static TOWN_ASSERT: Phrasings = Phrasings {
    train: &[
        "My hometown is {v}.",
        "I grew up in {v}.",
        "I am from {v}.",
        "I come from {v}.",
    ],
    development: &["I was raised in {v}.", "Home for me is {v}."],
};
static TOWN_UPDATE: Phrasings = Phrasings {
    train: &["Actually, my hometown is {v}.", "Sorry, I grew up in {v}."],
    development: &["Correction: my hometown is {v}."],
};
static TOWN_QUERY: Phrasings = Phrasings {
    train: &[
        "What is my hometown?",
        "Where did I grow up?",
        "Where am I from?",
    ],
    development: &["Remind me where I am from.", "Which town did I grow up in?"],
};
static NUMBER_ASSERT: Phrasings = Phrasings {
    train: &[
        "My lucky number is {v}.",
        "Remember, my lucky number is {v}.",
        "Please note that my lucky number is {v}.",
    ],
    development: &["{v} is my lucky number.", "The number {v} brings me luck."],
};
static NUMBER_UPDATE: Phrasings = Phrasings {
    train: &[
        "Actually, my lucky number is {v} now.",
        "Sorry, my lucky number is really {v}.",
    ],
    development: &["Correction: my lucky number is {v}."],
};
static NUMBER_QUERY: Phrasings = Phrasings {
    train: &[
        "What is my lucky number?",
        "What's my lucky number?",
        "Do you remember my lucky number?",
    ],
    development: &[
        "Remind me of my lucky number.",
        "Which number did I say brings me luck?",
    ],
};
static CODE_ASSERT: Phrasings = Phrasings {
    train: &[
        "My code word is {v}.",
        "The code word is {v}.",
        "Remember, the code word is {v}.",
    ],
    development: &["Our secret code word is {v}.", "Let the code word be {v}."],
};
static CODE_UPDATE: Phrasings = Phrasings {
    train: &[
        "Actually, the code word is {v} now.",
        "Sorry, the new code word is {v}.",
    ],
    development: &["Correction: the code word is {v}."],
};
static CODE_QUERY: Phrasings = Phrasings {
    train: &[
        "What is the code word?",
        "What's my code word?",
        "Do you remember the code word?",
    ],
    development: &[
        "What was the code word again?",
        "Remind me of the code word.",
    ],
};
static COLOR_ASSERT: Phrasings = Phrasings {
    train: &[
        "My favorite color is {v}.",
        "I love the color {v}.",
        "I like {v} best of all the colors.",
    ],
    development: &["The color I like most is {v}."],
};
static COLOR_UPDATE: Phrasings = Phrasings {
    train: &[
        "Actually, my favorite color is {v} now.",
        "I changed my mind, I like {v} best of the colors.",
    ],
    development: &["These days my favorite color is {v}."],
};
static COLOR_QUERY: Phrasings = Phrasings {
    train: &[
        "What is my favorite color?",
        "Which color do I like best?",
        "What color do I love?",
    ],
    development: &[
        "Remind me of my favorite color.",
        "What color did I say I like most?",
    ],
};

/// The names of the relation table's relations, in table order: open
/// relations first, then the closed minority. A relation turn's intent is
/// `{name}_assert`, `_update`, `_query` or (a query of an unstated relation)
/// `_absent`.
pub fn relation_names() -> Vec<&'static str> {
    relations().iter().map(|relation| relation.name).collect()
}

/// The relation table: open relations first, then the closed minority.
fn relations() -> &'static [Rel] {
    static TABLE: OnceLock<Vec<Rel>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let v1 = |index: usize, name: &'static str, values: Values| Rel {
            name,
            values,
            assert: &RELATIONS[index].assert,
            update: &RELATIONS[index].update,
            query: &RELATIONS[index].query,
            acks: RELATIONS[index].acks,
            answers: RELATIONS[index].answers,
        };
        let closed = |index: usize| {
            v1(
                index,
                RELATIONS[index].name,
                Values::Closed(
                    RELATIONS[index].train_values,
                    RELATIONS[index].development_values,
                ),
            )
        };
        vec![
            v1(0, "user_name", Values::UserName),
            v1(4, "pet_name", Values::PetName),
            Rel {
                name: "friend_name",
                values: Values::FriendName,
                assert: &FRIEND_ASSERT,
                update: &FRIEND_UPDATE,
                query: &FRIEND_QUERY,
                acks: &[
                    "Nice, {v} sounds like a good friend.",
                    "Got it, your friend is {v}.",
                ],
                answers: &["Your friend is {v}.", "Your friend's name is {v}."],
            },
            Rel {
                name: "hometown",
                values: Values::Hometown,
                assert: &TOWN_ASSERT,
                update: &TOWN_UPDATE,
                query: &TOWN_QUERY,
                acks: &[
                    "{v} sounds like an interesting place.",
                    "Got it, you grew up in {v}.",
                ],
                answers: &["Your hometown is {v}.", "You grew up in {v}."],
            },
            Rel {
                name: "lucky_number",
                values: Values::LuckyNumber,
                assert: &NUMBER_ASSERT,
                update: &NUMBER_UPDATE,
                query: &NUMBER_QUERY,
                acks: &[
                    "Got it, your lucky number is {v}.",
                    "Noted, {v} is lucky for you.",
                ],
                answers: &["Your lucky number is {v}.", "It is {v}."],
            },
            Rel {
                name: "code_word",
                values: Values::CodeWord,
                assert: &CODE_ASSERT,
                update: &CODE_UPDATE,
                query: &CODE_QUERY,
                acks: &[
                    "Got it, the code word is {v}.",
                    "Okay, {v} is the code word.",
                ],
                answers: &["The code word is {v}.", "It is {v}."],
            },
            closed(1),
            closed(2),
            closed(3),
            Rel {
                name: "favorite_color",
                values: Values::Closed(
                    &[
                        "red", "blue", "green", "yellow", "purple", "orange", "pink", "black",
                    ],
                    &["white", "brown", "gray"],
                ),
                assert: &COLOR_ASSERT,
                update: &COLOR_UPDATE,
                query: &COLOR_QUERY,
                acks: &["{v} is a pretty color.", "Got it, you love {v}."],
                answers: &["Your favorite color is {v}.", "You love {v}."],
            },
        ]
    })
}

const UPDATE_ACKS: &[&str] = &[
    "Okay, I'll remember that.",
    "Got it, thanks for telling me.",
];

// ---------------------------------------------------------------------------
// MQAR and copy templates.

/// "Please remember: {p}." where `{p}` is "bol is 47, tamir is kavu".
static MQAR_LEAD: Phrasings = Phrasings {
    train: &[
        "Please remember: {p}.",
        "Remember these: {p}.",
        "Keep these in mind: {p}.",
        "Note these down: {p}.",
    ],
    development: &[
        "I want you to memorize: {p}.",
        "Here is what to remember: {p}.",
        "Store these facts: {p}.",
    ],
};
/// No phrasing ends in "{k} is": the assertion states "{k} is {v}", so a query
/// ending in those two words is answered by copying what follows their latest
/// earlier occurrence (revision 2.1; R-nlet).
static MQAR_QUERY: Phrasings = Phrasings {
    train: &[
        "What is {k}?",
        "What was {k} again?",
        "Tell me what {k} was.",
        "Do you remember what {k} was?",
    ],
    development: &[
        "Remind me what {k} was.",
        "What did I tell you {k} was?",
        "Quick question: what is {k}?",
        "Say what {k} was.",
    ],
};
const MQAR_ACKS: &[&str] = &[
    "Okay, I'll remember all of that.",
    "Got it, I'll remember them.",
    "Noted.",
    "Okay, I have all of them.",
    "Got it.",
    "Thanks for telling me, I'll remember.",
];
const MQAR_REPLIES: &[&str] = &["{k} is {v}.", "{k} is {v}.", "It's {v}.", "That's {v}."];
/// World v2c's MQAR replies (Step 4, #820): index for index v2's, with each
/// rehearsal framed as "So {k} is {v}." so the key is never the reply's first
/// word. v2 capitalizes the reply's first character, which turns a rehearsed
/// key "luk" into "Luk", a token tuple that never occurs in the assertion or
/// the question (Step 0b, #1709: 41 of 41 rehearsals in the D19 draw). Under
/// the frame the key keeps its asserted casing and, as in the assertion and
/// the question, follows a space mid-sentence, under protocol 1 and 2 alike.
/// The bare forms do not state the key and are unchanged.
const MQAR_REPLIES_V2C: &[&str] = &[
    "So {k} is {v}.",
    "So {k} is {v}.",
    "It's {v}.",
    "That's {v}.",
];

/// Which M-world v2 table set a generator draws from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    /// The revision-2.1 world (#1511), byte-identical to every earlier draw.
    #[default]
    V2,
    /// v2 with [`MQAR_REPLIES_V2C`]: a rehearsed MQAR key keeps its asserted
    /// casing. Every RNG draw is v2's; the reply text differs only in the
    /// frame, so a draw differs from v2's where the one-token-longer reply
    /// changes a context fit (see [`MWorld2::with_variant`]).
    V2c,
}

impl Variant {
    /// The `world=` spelling: `v2` or `v2c`.
    pub fn key(self) -> &'static str {
        match self {
            Self::V2 => "v2",
            Self::V2c => "v2c",
        }
    }

    /// The `world` field reports record: `m-world-v2` or `m-world-v2c`.
    pub fn world_name(self) -> &'static str {
        match self {
            Self::V2 => "m-world-v2",
            Self::V2c => "m-world-v2c",
        }
    }

    /// Parse a `world=` value (`v2` or `v2c`).
    pub fn parse(text: &str) -> Result<Self> {
        match text {
            "v2" => Ok(Self::V2),
            "v2c" => Ok(Self::V2c),
            other => Err(invalid(format!("unknown v2 world {other:?} (v2 or v2c)"))),
        }
    }

    /// The variant a report's `world` field names; `m-world-v2` and an absent
    /// field are v2.
    pub fn from_world_name(name: Option<&str>) -> Result<Self> {
        match name {
            None | Some("m-world-v2") => Ok(Self::V2),
            Some("m-world-v2c") => Ok(Self::V2c),
            Some(other) => Err(invalid(format!("not an M-world v2 report: {other:?}"))),
        }
    }

    fn mqar_replies(self) -> &'static [&'static str] {
        match self {
            Self::V2 => MQAR_REPLIES,
            Self::V2c => MQAR_REPLIES_V2C,
        }
    }
}
static COPY: Phrasings = Phrasings {
    train: &[
        "Repeat exactly: {w}",
        "Say exactly this: {w}",
        "Copy these words: {w}",
        "Please repeat: {w}",
    ],
    development: &[
        "Say back word for word: {w}",
        "Echo this: {w}",
        "Write out exactly: {w}",
    ],
};

// ---------------------------------------------------------------------------
// Episodes.

/// The milestone category (v1's three) plus the two retrieval categories.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Category2 {
    Responsive,
    Relation,
    Instruction,
    Mqar,
    Copy,
}

impl From<Category> for Category2 {
    fn from(category: Category) -> Self {
        match category {
            Category::Responsive => Self::Responsive,
            Category::Relation => Self::Relation,
            Category::Instruction => Self::Instruction,
        }
    }
}

/// What kind of conversation an episode is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Kind {
    Mqar,
    Copy,
    Relation,
    Other,
}

/// An MQAR item's design and measurement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mqar {
    /// Pairs asserted, and pairs the cell asked for (fewer when the episode
    /// would not fit the context).
    pub n: usize,
    pub n_requested: usize,
    /// The distance bucket the item belongs to.
    pub target_distance: usize,
    /// Achieved tokens between the end of the assertion turn and the query.
    pub distance: usize,
    /// The same, counted from the end of the queried pair's value.
    pub pair_distance: usize,
    /// Which pair (0-based) the query asks for.
    pub queried: usize,
    /// The whole episode's tokens.
    pub tokens: usize,
}

/// What the evaluator reports a scored turn under, and the facts the rule
/// baselines and the teacher-forced scores read.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    /// Open or closed value pool of a relation or MQAR query.
    pub pool: Option<Pool>,
    /// The query asks for a relation that was never stated.
    pub abstain: bool,
    pub mqar: Option<Mqar>,
    /// The user phrasing template the turn was drawn from, slots unfilled
    /// (relation and MQAR queries, statements, copy and abstentions).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// The value a retrieval turn's reply must state (MQAR and relation
    /// recall); absent for copy and for an abstention.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    /// The open-pool values the turn's user text states, in reading order:
    /// the generator's recorded value spans. Closed values are not recorded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
}

/// One user turn, the reply the world trains on, and the v2 oracle's checks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn2 {
    pub intent: String,
    pub category: Category2,
    pub user: String,
    pub reply: String,
    pub checks: Vec<Check2>,
    pub tag: Tag,
}

/// A conversation of user turns and their replies, in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation2 {
    pub kind: Kind,
    pub turns: Vec<Turn2>,
    /// Tokens of the whole rendered document under the meter.
    pub tokens: usize,
}

/// The mix of conversation kinds.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mix {
    pub mqar: f64,
    pub copy: f64,
    pub relation: f64,
    pub other: f64,
    /// The fraction of relation episodes that use a closed relation.
    pub closed: f64,
}

impl Default for Mix {
    fn default() -> Self {
        Self {
            mqar: 0.35,
            copy: 0.10,
            relation: 0.30,
            other: 0.25,
            closed: CLOSED_SHARE,
        }
    }
}

impl Mix {
    fn validate(&self) -> Result<()> {
        let shares = [self.mqar, self.copy, self.relation, self.other];
        if shares.iter().any(|s| !s.is_finite() || *s < 0.0)
            || (shares.iter().sum::<f64>() - 1.0).abs() > 1e-9
            || !(0.0..=1.0).contains(&self.closed)
        {
            return Err(invalid(
                "mix shares must be nonnegative and sum to 1, and closed must be in 0..=1",
            ));
        }
        Ok(())
    }
}

/// Token counts in the caller's real tokenizer, laid out exactly as
/// `DialogueProtocol::literal_roles_v1` encodes a document: BOS, then for each
/// exchange "\n" (after the first), "User: ", the trimmed user text, "\n",
/// "Assistant: ", the trimmed reply and EOS. `count` is the number of tokens of
/// a text. [`Meter::spaced`] counts version 2's layout instead.
pub struct Meter<'a> {
    count: &'a dyn Fn(&str) -> usize,
    newline: usize,
    user: usize,
    assistant: usize,
    /// Version 2: markers end at the colon and each content carries its
    /// leading space.
    spaced: bool,
}

impl<'a> Meter<'a> {
    pub fn new(count: &'a dyn Fn(&str) -> usize) -> Self {
        Self {
            count,
            newline: count("\n"),
            user: count("User: "),
            assistant: count("Assistant: "),
            spaced: false,
        }
    }

    /// The meter of `DialogueProtocol::literal_roles_v2`'s layout: "User:"
    /// and "Assistant:", each followed by " " and the trimmed content.
    pub fn spaced(count: &'a dyn Fn(&str) -> usize) -> Self {
        Self {
            count,
            newline: count("\n"),
            user: count("User:"),
            assistant: count("Assistant:"),
            spaced: true,
        }
    }

    /// Tokens of a content text.
    pub fn text(&self, text: &str) -> usize {
        if self.spaced {
            (self.count)(&format!(" {}", text.trim()))
        } else {
            (self.count)(text.trim())
        }
    }

    /// Tokens of the exchange at position `index` of a document.
    pub fn exchange(&self, index: usize, user: &str, reply: &str) -> usize {
        usize::from(index != 0) * self.newline
            + self.user
            + self.text(user)
            + self.newline
            + self.assistant
            + self.text(reply)
            + 1
    }

    /// Tokens of a whole document of these turns.
    pub fn document(&self, turns: &[Turn2]) -> usize {
        1 + turns
            .iter()
            .enumerate()
            .map(|(i, t)| self.exchange(i, &t.user, &t.reply))
            .sum::<usize>()
    }

    /// Tokens between the last token of turn 0's user text and the first
    /// token of the last turn's user text.
    pub fn distance(&self, turns: &[Turn2]) -> usize {
        let (Some(first), true) = (turns.first(), turns.len() >= 2) else {
            return 0;
        };
        let middle = &turns[1..turns.len() - 1];
        self.newline
            + self.assistant
            + self.text(&first.reply)
            + 1
            + middle
                .iter()
                .enumerate()
                .map(|(i, t)| self.exchange(i + 1, &t.user, &t.reply))
                .sum::<usize>()
            + self.newline
            + self.user
    }
}

fn unit(rng: &mut Rng) -> f64 {
    (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64
}

/// A v1 turn under the v2 oracle, with v1's article rule applied.
fn lift(turn: Turn) -> Turn2 {
    Turn2 {
        intent: turn.intent,
        category: turn.category.into(),
        user: articles(&turn.user),
        reply: articles(&turn.reply),
        checks: turn.checks.iter().map(Check2::from_v1).collect(),
        tag: Tag::default(),
    }
}

fn ack_checks() -> Vec<Check2> {
    vec![Check2::V1(Check::AnyOf(strings(ACK_WORDS)))]
}

/// The M-world v2 generator. Carries the MQAR cell counter, so a stream of
/// episodes cycles through every (D, N) cell evenly.
pub struct MWorld2<'a> {
    meter: Meter<'a>,
    mix: Mix,
    mqar_seen: usize,
    variant: Variant,
}

impl<'a> MWorld2<'a> {
    pub fn new(count: &'a dyn Fn(&str) -> usize, mix: Mix) -> Result<Self> {
        mix.validate()?;
        Ok(Self {
            meter: Meter::new(count),
            mix,
            mqar_seen: 0,
            variant: Variant::V2,
        })
    }

    /// The same generator drawing from `variant`'s tables. The MQAR reply is
    /// chosen by v2's template lengths and then written from the variant's
    /// table at the same index, so a seed makes the same choices in v2 and
    /// v2c; the draws differ only where the longer v2c reply changes whether
    /// an episode fits the context.
    pub fn with_variant(mut self, variant: Variant) -> Self {
        self.variant = variant;
        self
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn meter(&self) -> &Meter<'a> {
        &self.meter
    }

    pub fn mix(&self) -> Mix {
        self.mix
    }

    /// One episode of `split`: the kind is drawn from the mix, and every
    /// reference reply passes its own checks and fits [`CONTEXT`] tokens.
    pub fn conversation(&mut self, rng: &mut Rng, split: Split) -> Result<Conversation2> {
        self.conversation_in(rng, Cell::same(split))
    }

    /// One episode of `cell`: its phrasings from `cell.phrasing`, its values
    /// from `cell.value`. Under [`Cell::same`] this is [`Self::conversation`].
    pub fn conversation_in(&mut self, rng: &mut Rng, cell: Cell) -> Result<Conversation2> {
        for _ in 0..64 {
            let u = unit(rng);
            let (kind, turns) = if u < self.mix.mqar {
                (Kind::Mqar, self.mqar(rng, cell)?)
            } else if u < self.mix.mqar + self.mix.copy {
                (Kind::Copy, copy(rng, cell)?)
            } else if u < self.mix.mqar + self.mix.copy + self.mix.relation {
                (Kind::Relation, self.relation(rng, cell)?)
            } else {
                (Kind::Other, other(rng, cell.phrasing))
            };
            let tokens = self.meter.document(&turns);
            let sound = turns.iter().all(|t| judge_v2(&t.checks, &t.user, &t.reply));
            if sound && tokens <= CONTEXT {
                return Ok(Conversation2 {
                    kind,
                    turns,
                    tokens,
                });
            }
        }
        Err(invalid("no M-world v2 episode fits the context"))
    }

    /// An episode none of whose user turns equals an `excluded` turn (compared
    /// by [`crate::milestone_world::normalized`]); rejected draws are counted.
    pub fn conversation_excluding(
        &mut self,
        rng: &mut Rng,
        split: Split,
        excluded: &BTreeSet<String>,
        rejected: &mut usize,
    ) -> Result<Conversation2> {
        for _ in 0..10_000 {
            let conversation = self.conversation(rng, split)?;
            if conversation
                .turns
                .iter()
                .all(|t| !excluded.contains(&crate::milestone_world::normalized(&t.user)))
            {
                return Ok(conversation);
            }
            *rejected += 1;
        }
        Err(invalid(
            "the exclusion list rejects every M-world v2 conversation",
        ))
    }

    /// The MQAR episode of the next (D, N) cell: N pairs asserted in one turn
    /// (fewer when the cell's N would not fit), acknowledged, then responsive
    /// filler turns until the distance reaches D, then the query.
    fn mqar(&mut self, rng: &mut Rng, cell: Cell) -> Result<Vec<Turn2>> {
        let slot = self.mqar_seen;
        self.mqar_seen += 1;
        let distance = DISTANCES[slot % DISTANCES.len()];
        let requested = PAIR_COUNTS[(slot / DISTANCES.len()) % PAIR_COUNTS.len()];
        let mut n = requested;
        loop {
            // Ordinary draws first; then compact ones (two-syllable words, the
            // shortest templates), which are what fits a long distance.
            for attempt in 0..60 {
                let compact = attempt >= 10;
                if let Some(turns) = self.try_mqar(rng, cell, (n, requested), distance, compact)? {
                    return Ok(turns);
                }
            }
            if n <= PAIR_COUNTS[0] {
                return Err(invalid(format!(
                    "no MQAR episode of {n} pairs reaches {distance} tokens within the context"
                )));
            }
            n /= 2;
        }
    }

    fn try_mqar(
        &self,
        rng: &mut Rng,
        cell: Cell,
        (n, requested): (usize, usize),
        target: usize,
        compact: bool,
    ) -> Result<Option<Vec<Turn2>>> {
        let syllables = if compact { (2, 2) } else { (2, 3) };
        // 2n distinct strings: keys are lowercase words, values numbers or words.
        let mut used: BTreeSet<String> = BTreeSet::new();
        let (mut keys, mut values) = (Vec::new(), Vec::new());
        for _ in 0..n {
            for is_key in [true, false] {
                loop {
                    let candidate = if is_key || rng.below(2) == 0 {
                        word(rng, cell.value, syllables.0, syllables.1)?
                    } else {
                        digits(rng, cell.value)?
                    };
                    if used.insert(candidate.clone()) {
                        if is_key {
                            keys.push(candidate);
                        } else {
                            values.push(candidate);
                        }
                        break;
                    }
                }
            }
        }
        let queried = rng.below(n);
        let pairs: Vec<String> = keys
            .iter()
            .zip(&values)
            .map(|(k, v)| format!("{k} is {v}"))
            .collect();
        // A template of the phrasing split; a compact draw takes the shortest
        // of four.
        let template = |rng: &mut Rng, phrasings: &Phrasings| -> &'static str {
            let draws = if compact { 4 } else { 1 };
            (0..draws)
                .map(|_| phrasings.pick(rng, cell.phrasing))
                .min_by_key(|t| self.meter.text(t))
                .unwrap_or_default()
        };
        let lead = template(rng, &MQAR_LEAD);
        let assertion = fill(lead, &[("p", &pairs.join(", "))]);
        let ack = *pick(rng, MQAR_ACKS);
        let (key, value) = (&keys[queried], &values[queried]);
        let query_template = template(rng, &MQAR_QUERY);
        let mut query = fill(query_template, &[("k", key)]);
        capitalize(&mut query);
        // The choice is made on v2's table (as v2 always made it), then
        // written from the variant's table at the same index.
        let reply_index = (0..if compact { 4 } else { 1 })
            .map(|_| rng.below(MQAR_REPLIES.len()))
            .min_by_key(|&i| self.meter.text(MQAR_REPLIES[i]))
            .unwrap_or_default();
        let reply = self.variant.mqar_replies()[reply_index];
        let mut answer = fill(reply, &[("k", key), ("v", value)]);
        capitalize(&mut answer);
        let others: Vec<String> = values
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != queried)
            .map(|(_, v)| v.to_lowercase())
            .collect();
        let mut turns = vec![Turn2 {
            intent: "mqar_assert".into(),
            category: Category2::Responsive,
            user: assertion.clone(),
            reply: ack.into(),
            checks: ack_checks(),
            tag: Tag {
                values: values.clone(),
                ..Tag::default()
            },
        }];
        // Everything but the distance: BOS, the assertion, the query and its
        // answer. What is left of the context bounds the distance.
        let fixed = 1
            + self.meter.user
            + self.meter.text(&assertion)
            + self.meter.text(&query)
            + self.meter.newline
            + self.meter.assistant
            + self.meter.text(&answer)
            + 1;
        let ceiling = (target + tolerance(target)).min(CONTEXT.saturating_sub(fixed));
        if ceiling < target {
            return Ok(None);
        }
        // Filler: responsive turns, drawn until the distance reaches the
        // target without passing the ceiling.
        let mut distance = self.meter.newline
            + self.meter.assistant
            + self.meter.text(ack)
            + 1
            + self.meter.newline
            + self.meter.user;
        let mut guard = 0;
        while distance < target {
            guard += 1;
            if guard > 64 {
                return Ok(None);
            }
            let (mut finishing, mut progress) = (Vec::new(), Vec::new());
            for _ in 0..8 {
                let turn = loop {
                    let turn = lift(responsive(rng, cell.phrasing));
                    if turn.intent != "farewell" {
                        break turn;
                    }
                };
                let cost = self.meter.exchange(turns.len(), &turn.user, &turn.reply);
                if distance + cost >= target {
                    if distance + cost <= ceiling {
                        finishing.push((turn, cost));
                    }
                } else {
                    progress.push((turn, cost));
                }
            }
            let chosen = if finishing.is_empty() {
                &mut progress
            } else {
                &mut finishing
            };
            if chosen.is_empty() {
                continue;
            }
            let (turn, cost) = chosen.swap_remove(rng.below(chosen.len()));
            distance += cost;
            turns.push(turn);
        }
        if distance > ceiling {
            return Ok(None);
        }
        turns.push(Turn2 {
            intent: "mqar_query".into(),
            category: Category2::Mqar,
            user: query,
            reply: answer,
            checks: vec![
                Check2::V1(Check::AnyOf(vec![value.to_lowercase()])),
                Check2::V1(Check::NoneOf(others)),
            ],
            tag: Tag::default(),
        });
        for turn in &mut turns {
            turn.user = articles(&turn.user);
            turn.reply = articles(&turn.reply);
        }
        let tokens = self.meter.document(&turns);
        if tokens > CONTEXT {
            return Ok(None);
        }
        let measured = self.meter.distance(&turns);
        if measured < target || measured > ceiling {
            return Ok(None);
        }
        // Where the queried pair's value ends in the assertion text.
        let lead_len = lead.find("{p}").unwrap_or(0);
        let assertion_end =
            lead_len + pairs[..=queried].iter().map(String::len).sum::<usize>() + 2 * queried;
        let tail = self
            .meter
            .text(&assertion)
            .saturating_sub(self.meter.text(&assertion[..assertion_end]));
        if let Some(last) = turns.last_mut() {
            last.tag = Tag {
                pool: Some(Pool::Open),
                abstain: false,
                mqar: Some(Mqar {
                    n,
                    n_requested: requested,
                    target_distance: target,
                    distance: measured,
                    pair_distance: measured + tail,
                    queried,
                    tokens,
                }),
                template: Some(query_template.to_owned()),
                answer: Some(value.clone()),
                values: Vec::new(),
            };
        }
        Ok(Some(turns))
    }

    /// Assert, an optional filler and update, and the assertion of a companion
    /// relation of the same pool, then the query about one of the two (or an
    /// abstention about a relation stated nowhere), with open values for the
    /// open relations.
    ///
    /// Which of the two stated relations the query asks is a fair coin drawn
    /// from the episode's own seeded stream: deterministic for the seed and
    /// balanced, and independent of the order the statements come in, so the
    /// stated values in themselves never tell the answer. With probability
    /// [`DISTRACTOR`] the companion is stated after the queried relation's
    /// update; otherwise (with an update) the update comes last. A closed
    /// relation takes a closed companion and an open one an open companion, so
    /// the pool of the asked relation is independent of which one was asked.
    fn relation(&self, rng: &mut Rng, cell: Cell) -> Result<Vec<Turn2>> {
        let table = relations();
        let wanted = if unit(rng) < self.mix.closed {
            Pool::Closed
        } else {
            Pool::Open
        };
        let candidates: Vec<&Rel> = table.iter().filter(|r| r.pool() == wanted).collect();
        let rel = *pick(rng, &candidates);
        let first = rel.draw(rng, cell.value)?;
        let mut turns = vec![rel_turn(
            rng,
            cell.phrasing,
            rel,
            Act::Assert,
            &first,
            ack_with(&first),
            rel.acks,
        )];
        if rng.below(2) == 0 {
            turns.push(lift(responsive(rng, cell.phrasing)));
        }
        let mut current = first.clone();
        let mut update: Option<Turn2> = None;
        if rng.below(2) == 0 {
            let mut second = rel.draw(rng, cell.value)?;
            while second == first {
                second = rel.draw(rng, cell.value)?;
            }
            update = Some(rel_turn(
                rng,
                cell.phrasing,
                rel,
                Act::Update,
                &second,
                ack_with(&second),
                UPDATE_ACKS,
            ));
            current = second;
        }
        let updated = update.is_some();
        // The companion: another relation of the same pool, stated once.
        let peers: Vec<&Rel> = table
            .iter()
            .filter(|r| r.pool() == wanted && r.name != rel.name)
            .collect();
        let companion = *pick(rng, &peers);
        let mut companion_value = companion.draw(rng, cell.value)?;
        while companion_value.eq_ignore_ascii_case(&first)
            || companion_value.eq_ignore_ascii_case(&current)
        {
            companion_value = companion.draw(rng, cell.value)?;
        }
        let companion_turn = rel_turn(
            rng,
            cell.phrasing,
            companion,
            Act::Assert,
            &companion_value,
            ack_with(&companion_value),
            companion.acks,
        );
        let companion_last = rng.below(DISTRACTOR.1) < DISTRACTOR.0;
        match update {
            Some(update) if companion_last => {
                turns.push(update);
                turns.push(companion_turn);
            }
            Some(update) => {
                turns.push(companion_turn);
                turns.push(update);
            }
            None => turns.push(companion_turn),
        }
        if rng.below(4) == 0 {
            let others: Vec<&Rel> = table
                .iter()
                .filter(|r| r.name != rel.name && r.name != companion.name)
                .collect();
            let asked = *pick(rng, &others);
            let mut rejected: BTreeSet<String> = [&first, &current, &companion_value]
                .into_iter()
                .map(|v| v.to_lowercase())
                .collect();
            if let Values::Closed(train, development) = asked.values {
                rejected.extend(train.iter().chain(development).map(|v| v.to_lowercase()));
            }
            let template = asked.query.pick(rng, cell.phrasing);
            turns.push(Turn2 {
                intent: format!("{}_absent", asked.name),
                category: Category2::Relation,
                user: template.into(),
                reply: (*pick(rng, ABSENT_REPLIES)).into(),
                checks: vec![
                    Check2::V1(Check::AnyOf(strings(ABSENT_ACCEPT))),
                    Check2::V1(Check::NoneOf(rejected.into_iter().collect())),
                ],
                tag: Tag {
                    pool: Some(asked.pool()),
                    abstain: true,
                    template: Some(template.to_owned()),
                    ..Tag::default()
                },
            });
        } else {
            // The coin that decides which relation is asked; the values of the
            // other relation (and the queried relation's replaced value) are
            // the ones the reply must not state.
            let ask_companion = rng.below(2) == 0;
            let (asked, answer, stale) = if ask_companion {
                let mut stale = vec![first.to_lowercase()];
                if updated {
                    stale.push(current.to_lowercase());
                }
                (companion, companion_value.clone(), stale)
            } else {
                let mut stale = vec![companion_value.to_lowercase()];
                if updated {
                    stale.push(first.to_lowercase());
                }
                (rel, current.clone(), stale)
            };
            let checks = vec![
                Check2::V1(Check::AnyOf(vec![answer.to_lowercase()])),
                Check2::V1(Check::NoneOf(stale)),
            ];
            let mut query = rel_turn(
                rng,
                cell.phrasing,
                asked,
                Act::Query,
                &answer,
                checks,
                asked.answers,
            );
            query.tag.pool = Some(asked.pool());
            query.tag.answer = Some(answer);
            turns.push(query);
        }
        for turn in &mut turns {
            turn.user = articles(&turn.user);
            turn.reply = articles(&turn.reply);
        }
        Ok(turns)
    }

    /// Every user phrasing template of `split` that v2 can render, for
    /// disjointness checks: v1's, and v2's MQAR, copy and relation templates.
    pub fn templates(split: Split) -> Vec<&'static str> {
        let mut all = MWorld::templates(split);
        let mut add = |p: &Phrasings| {
            all.extend(match split {
                Split::Train => p.train,
                Split::Development => p.development,
            })
        };
        add(&MQAR_LEAD);
        add(&MQAR_QUERY);
        add(&COPY);
        for rel in relations() {
            add(rel.assert);
            add(rel.update);
            add(rel.query);
        }
        all
    }

    /// SHA-256 of every table v2 draws from, the revision, the relation
    /// distractor share ([`DISTRACTOR`], under its original key), the cell
    /// names, the judge's version and v1's digest (v2 reuses v1's responsive
    /// and instruction tables). Generator code is not in it; see the tests'
    /// `STREAM_DIGEST` for the stream a fixed seed draws.
    pub fn digest() -> String {
        let phrasings = |p: &Phrasings| json!([p.train, p.development]);
        let tables = json!({
            "version": "m-world-v2",
            "revision": REVISION,
            "judge": "judge2-1",
            "v1_digest": MWorld::digest(),
            "context": CONTEXT,
            "distances": DISTANCES,
            "pair_counts": PAIR_COUNTS,
            "tolerance": "max(distance/8, 8)",
            "split": [SPLIT_KEY, DEVELOPMENT_ONE_IN],
            "syllables": [ONSETS, VOWELS, CODAS],
            "town_suffixes": TOWN_SUFFIXES,
            "mix": Mix::default(),
            "mqar": [phrasings(&MQAR_LEAD), phrasings(&MQAR_QUERY), MQAR_ACKS, MQAR_REPLIES],
            "copy": [phrasings(&COPY), 3, 8],
            "update_acks": UPDATE_ACKS,
            "relation_distractor": [DISTRACTOR.0, DISTRACTOR.1],
            "cells": Cell::ALL.map(Cell::key),
            "relations": relations().iter().map(|r| {
                let closed = match r.values {
                    Values::Closed(train, development) => json!([train, development]),
                    _ => Value::Null,
                };
                json!([r.name, format!("{:?}", r.pool()), closed, phrasings(r.assert), phrasings(r.update), phrasings(r.query), r.acks, r.answers])
            }).collect::<Vec<_>>(),
        });
        hex::encode(Sha256::digest(tables.to_string().as_bytes()))
    }

    /// The digest of `variant`'s tables: [`Self::digest`] for v2, unchanged;
    /// for v2c the SHA-256 of v2's digest, the variant's name and its MQAR
    /// reply table. ("So" is one syllable, so it cannot collide with a
    /// generated word, and the reserved words stay v2's.)
    pub fn digest_for(variant: Variant) -> String {
        match variant {
            Variant::V2 => Self::digest(),
            Variant::V2c => {
                let tables = json!({
                    "base": Self::digest(),
                    "variant": variant.world_name(),
                    "mqar_replies": MQAR_REPLIES_V2C,
                    "reply_choice": "v2 template lengths, same index",
                });
                hex::encode(Sha256::digest(tables.to_string().as_bytes()))
            }
        }
    }
}

/// Which part of a relation conversation a turn is.
#[derive(Clone, Copy)]
enum Act {
    Assert,
    Update,
    Query,
}

fn ack_with(value: &str) -> Vec<Check2> {
    let mut accepted = strings(ACK_WORDS);
    accepted.push(value.to_lowercase());
    vec![Check2::V1(Check::AnyOf(accepted))]
}

/// One turn of a relation conversation. `split` is the phrasing split; the
/// turn records its template and, for a statement of an open relation, the
/// value it states.
fn rel_turn(
    rng: &mut Rng,
    split: Split,
    rel: &Rel,
    act: Act,
    value: &str,
    checks: Vec<Check2>,
    replies: &[&str],
) -> Turn2 {
    let slots = [("v", value)];
    let (phrasings, suffix, category) = match act {
        Act::Assert => (rel.assert, "assert", Category2::Responsive),
        Act::Update => (rel.update, "update", Category2::Responsive),
        Act::Query => (rel.query, "query", Category2::Relation),
    };
    let template = phrasings.pick(rng, split);
    let mut tag = Tag {
        template: Some(template.to_owned()),
        ..Tag::default()
    };
    if matches!(act, Act::Assert | Act::Update) && rel.pool() == Pool::Open {
        tag.values = vec![value.to_owned()];
    }
    Turn2 {
        intent: format!("{}_{suffix}", rel.name),
        category,
        user: fill(template, &slots),
        reply: fill(pick(rng, replies), &slots),
        checks,
        tag,
    }
}

/// "Repeat exactly: w1 ... wk" (k from 3 to 8) and the same words back.
fn copy(rng: &mut Rng, cell: Cell) -> Result<Vec<Turn2>> {
    let count = 3 + rng.below(6);
    let mut list: Vec<String> = Vec::new();
    while list.len() < count {
        let candidate = word(rng, cell.value, 2, 3)?;
        if !list.contains(&candidate) {
            list.push(candidate);
        }
    }
    let joined = list.join(" ");
    let template = COPY.pick(rng, cell.phrasing);
    Ok(vec![Turn2 {
        intent: "copy".into(),
        category: Category2::Copy,
        user: fill(template, &[("w", &joined)]),
        reply: joined,
        checks: vec![Check2::CopyExact(list.clone())],
        tag: Tag {
            template: Some(template.to_owned()),
            values: list,
            ..Tag::default()
        },
    }])
}

/// v1's responsive and instruction intents in v1's conversation shape: an
/// optional greeting, one to three body turns, an optional thanks or farewell.
fn other(rng: &mut Rng, split: Split) -> Vec<Turn2> {
    let mut turns = Vec::new();
    if rng.below(3) == 0 {
        turns.push(lift(opener(rng, split)));
    }
    let instructions = rng.below(5) >= 2;
    for _ in 0..1 + rng.below(3) {
        turns.push(lift(if instructions {
            instruction(rng, split)
        } else {
            responsive(rng, split)
        }));
    }
    if rng.below(3) == 0 {
        turns.push(lift(closer(rng, split)));
    }
    turns
}

/// "User: ...\nAssistant: ..." per turn, for reading an episode.
pub fn render(turns: &[Turn2]) -> String {
    turns
        .iter()
        .map(|t| format!("User: {}\nAssistant: {}", t.user, t.reply))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Scoring.

/// Pass counts by category, intent, MQAR cell, copy, and open/closed relation
/// recall, and the A1 gate they decide.
#[derive(Default)]
pub struct Scorecard {
    cells: BTreeMap<String, (usize, usize)>,
}

impl Scorecard {
    fn add(&mut self, key: String, pass: bool) {
        let cell = self.cells.entry(key).or_default();
        cell.0 += usize::from(pass);
        cell.1 += 1;
    }

    /// Count one judged turn.
    pub fn record(&mut self, turn: &Turn2, pass: bool) {
        self.add(format!("category/{:?}", turn.category), pass);
        self.add(format!("intent/{}", turn.intent), pass);
        if let Some(mqar) = &turn.tag.mqar {
            self.add(format!("mqar/distance/{}", mqar.target_distance), pass);
            self.add(format!("mqar/n/{}", mqar.n), pass);
            self.add(format!("mqar/n_requested/{}", mqar.n_requested), pass);
            self.add(
                format!("mqar/cell/D{}xN{}", mqar.target_distance, mqar.n),
                pass,
            );
            self.add("mqar/all".into(), pass);
        }
        if turn.category == Category2::Copy {
            self.add("copy".into(), pass);
        }
        if turn.category == Category2::Relation {
            let pool = match turn.tag.pool {
                Some(Pool::Open) => "open",
                Some(Pool::Closed) => "closed",
                None => return,
            };
            let kind = if turn.tag.abstain {
                "abstain"
            } else {
                "recall"
            };
            self.add(format!("relation/{pool}/{kind}"), pass);
        }
    }

    fn cell(&self, key: &str) -> (usize, usize) {
        self.cells.get(key).copied().unwrap_or_default()
    }

    /// Pass rate of a cell; 0 for an empty one.
    pub fn rate(&self, key: &str) -> f64 {
        let (pass, of) = self.cell(key);
        if of == 0 {
            0.0
        } else {
            pass as f64 / of as f64
        }
    }

    /// The A1 gate: MQAR recall of at least 0.9 at every distance and
    /// recall of at least 0.9 on open-pool relation queries with a stated
    /// value (abstentions are reported apart). An empty cell fails.
    pub fn a1_gate(&self) -> bool {
        let passes = |key: &str| self.cell(key).1 > 0 && self.rate(key) >= 0.9;
        DISTANCES
            .iter()
            .all(|d| passes(&format!("mqar/distance/{d}")))
            && passes("relation/open/recall")
    }

    fn table(&self, prefix: &str) -> BTreeMap<String, Value> {
        self.cells
            .iter()
            .filter_map(|(key, &(pass, of))| {
                key.strip_prefix(prefix).map(|rest| {
                    (
                        rest.to_owned(),
                        json!({"pass": pass, "of": of, "rate": if of == 0 { 0.0 } else { pass as f64 / of as f64 }}),
                    )
                })
            })
            .collect()
    }

    /// The report's scores.
    pub fn to_json(&self) -> Value {
        let one = |key: &str| {
            let (pass, of) = self.cell(key);
            json!({"pass": pass, "of": of, "rate": self.rate(key)})
        };
        json!({
            "by_category": self.table("category/"),
            "by_intent": self.table("intent/"),
            "mqar": {
                "all": one("mqar/all"),
                "by_distance": self.table("mqar/distance/"),
                "by_n": self.table("mqar/n/"),
                "by_n_requested": self.table("mqar/n_requested/"),
                "by_cell": self.table("mqar/cell/"),
            },
            "copy": one("copy"),
            "relation": {
                "open": one("relation/open/recall"),
                "closed": one("relation/closed/recall"),
                "open_abstain": one("relation/open/abstain"),
                "closed_abstain": one("relation/closed/abstain"),
            },
            "a1_gate": self.a1_gate(),
            "a1_gate_rule": A1_GATE_RULE,
        })
    }
}

/// The frozen A1 criterion, as every report words it.
const A1_GATE_RULE: &str =
    "MQAR recall >= 0.9 at every distance (16, 64, 200) AND open-relation recall >= 0.9; an empty cell fails";

// ---------------------------------------------------------------------------
// Cross cells.

/// A turn the retrieval instrument scores: an MQAR query, a relation query
/// (recall or abstention) or a copy.
pub fn is_retrieval(turn: &Turn2) -> bool {
    matches!(
        turn.category,
        Category2::Mqar | Category2::Copy | Category2::Relation
    )
}

/// The headline rates of one scorecard: MQAR recall by distance and overall,
/// relation recall and abstention on each pool, and copy.
fn headline(card: &Scorecard) -> Value {
    let mut keys: Vec<String> = DISTANCES
        .iter()
        .map(|d| format!("mqar/distance/{d}"))
        .collect();
    keys.extend(
        [
            "mqar/all",
            "relation/open/recall",
            "relation/closed/recall",
            "relation/open/abstain",
            "relation/closed/abstain",
            "copy",
        ]
        .map(str::to_owned),
    );
    let rows: BTreeMap<String, Value> = keys
        .into_iter()
        .map(|key| {
            let (pass, of) = card.cell(&key);
            let row = json!({"pass": pass, "of": of, "rate": card.rate(&key)});
            (key, row)
        })
        .collect();
    json!(rows)
}

/// Scorecards of the four cells. The A1 gate is decided on the development
/// cell only ([`Cell::GATED`]), by the same [`Scorecard::a1_gate`] as before.
#[derive(Default)]
pub struct CellScores {
    cards: BTreeMap<Cell, Scorecard>,
}

impl CellScores {
    /// Count one judged turn under `cell`.
    pub fn record(&mut self, cell: Cell, turn: &Turn2, pass: bool) {
        self.cards.entry(cell).or_default().record(turn, pass);
    }

    /// The scorecard of `cell`, if any turn was counted under it.
    pub fn card(&self, cell: Cell) -> Option<&Scorecard> {
        self.cards.get(&cell)
    }

    /// The A1 gate: [`Scorecard::a1_gate`] of the development cell, false when
    /// that cell was not scored.
    pub fn a1_gate(&self) -> bool {
        self.card(Cell::GATED).is_some_and(Scorecard::a1_gate)
    }

    /// Every cell's scores by category, MQAR distance and pool, the headline
    /// matrix, and the pure-retrieval cell on its own. Only the development
    /// cell carries a gate.
    pub fn to_json(&self) -> Value {
        let mut cells: BTreeMap<&str, Value> = BTreeMap::new();
        let mut matrix: BTreeMap<&str, Value> = BTreeMap::new();
        for (cell, card) in &self.cards {
            let mut scores = card.to_json();
            if *cell != Cell::GATED {
                if let Some(object) = scores.as_object_mut() {
                    object.remove("a1_gate");
                    object.remove("a1_gate_rule");
                }
            }
            cells.insert(cell.key(), scores);
            matrix.insert(cell.key(), headline(card));
        }
        json!({
            "cells": cells,
            "matrix": matrix,
            "pure_retrieval": {
                "cell": Cell::PURE_RETRIEVAL.key(),
                "reading": "trained phrasing, values never seen: what is left of a failure once the phrasing is held fixed",
                "scores": self.cards.get(&Cell::PURE_RETRIEVAL).map(headline),
            },
            "gated_cell": Cell::GATED.key(),
            "a1_gate": self.a1_gate(),
            "a1_gate_rule": A1_GATE_RULE,
        })
    }
}

// ---------------------------------------------------------------------------
// The untrained rule baselines (the council's A1 amendment).

/// What a rule replies when it has nothing to copy.
pub const DONT_KNOW: &str = "I don't know.";

/// The freeze limit as a fraction: a rule must score strictly below 3/5 on
/// every gated cell for the instrument to freeze.
pub const FREEZE_LIMIT: (usize, usize) = (3, 5);

/// The row that reports the world's own reference replies through the oracle:
/// 1.0 unless the harness is broken.
pub const REFERENCE: &str = "reference";

/// An untrained rule over the reference history before the query. Words are
/// compared case-insensitively and without punctuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// R-recency: reply with the most recent open-pool value the generator
    /// recorded in the history ([`Tag::values`]).
    Recency,
    /// R-nlet: find the latest earlier occurrence of the query's last two
    /// words and reply with the words that follow it up to the clause end;
    /// with no such occurrence, reply "I don't know.".
    Nlet,
    /// R-sieve: the untrained identity channel of the log-sieve design
    /// (`docs/integration/log-sieve-retrieval-design-2026-10-01.md` §2.2,
    /// E1), [`sieve_value`]: "It's {value}." or "I don't know.". It is a
    /// retrieval route under test, **not** one of the instrument's freeze rules
    /// ([`Rule::ALL`]), which it is built to pass.
    Sieve,
}

impl Rule {
    /// The instrument's freeze rules: [`freeze_report`] and the probe read
    /// these alone.
    pub const ALL: [Rule; 2] = [Rule::Recency, Rule::Nlet];
    /// Retrieval routes scored beside the reference ([`run_route`]).
    pub const ROUTES: [Rule; 1] = [Rule::Sieve];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Recency => "R-recency",
            Self::Nlet => "R-nlet",
            Self::Sieve => "R-sieve",
        }
    }

    /// The rule's reply to `query` after `history`, the turns before it.
    pub fn reply(self, history: &[Turn2], query: &Turn2) -> String {
        match self {
            Self::Recency => history
                .iter()
                .rev()
                .find_map(|turn| turn.tag.values.last().cloned())
                .unwrap_or_else(|| DONT_KNOW.to_owned()),
            Self::Nlet => nlet_reply(history, query),
            Self::Sieve => match sieve_value(history, query) {
                Some(value) => format!("It's {value}."),
                None => DONT_KNOW.to_owned(),
            },
        }
    }
}

/// The untrained identity channel of the log-sieve design (§2.2 of
/// `docs/integration/log-sieve-retrieval-design-2026-10-01.md`): the value
/// an exact log of the user turns in `history` gives for `query`, or `None`.
///
/// - **Log.** Each earlier *user* turn, split into clauses at punctuation;
///   replies are the model's own and are not logged as facts.
/// - **Atoms.** The query's content words: its words outside the world's fixed
///   vocabulary ([`reserved_words`], instrument knowledge that a learned stop
///   list stands in for in English).
/// - **Admission.** A clause sharing at least one query atom (exactly
///   `gcd > 1` of the two squarefree prime products, computed as a set
///   intersection).
/// - **Ranking**, separate from admission: more shared atoms, then the latest
///   clause, so the latest version wins.
/// - **Value.** The words after the clause's first query atom up to the clause
///   end, without a leading copula; `None` if nothing follows.
pub fn sieve_value(history: &[Turn2], query: &Turn2) -> Option<String> {
    let log: Vec<&str> = history.iter().map(|turn| turn.user.as_str()).collect();
    sieve_value_text(&log, &query.user)
}

/// [`sieve_value`] over plain user-turn texts (oldest first) and a query
/// text: the same log, atoms, admission, ranking and value.
pub fn sieve_value_text(history: &[&str], query: &str) -> Option<String> {
    let reserved = reserved_words();
    let atoms: BTreeSet<String> = words(query)
        .into_iter()
        .filter(|word| !reserved.contains(word))
        .collect();
    if atoms.is_empty() {
        return None;
    }
    // (shared atoms, turn, clause) of the best clause so far, and its words.
    let mut best: Option<((usize, usize, usize), Vec<String>)> = None;
    for (turn_index, turn) in history.iter().enumerate() {
        let mut clause = Vec::new();
        let mut clause_index = 0;
        for (word, end) in clause_words(turn) {
            clause.push(word);
            if !end {
                continue;
            }
            let shared = clause
                .iter()
                .filter(|word| atoms.contains(*word))
                .collect::<BTreeSet<_>>()
                .len();
            let rank = (shared, turn_index, clause_index);
            if shared > 0 && best.as_ref().is_none_or(|(top, _)| rank > *top) {
                best = Some((rank, std::mem::take(&mut clause)));
            }
            clause.clear();
            clause_index += 1;
        }
    }
    let (_, clause) = best?;
    let at = clause.iter().position(|word| atoms.contains(word))?;
    let mut value = &clause[at + 1..];
    if value
        .first()
        .is_some_and(|word| matches!(word.as_str(), "is" | "was" | "are" | "were"))
    {
        value = &value[1..];
    }
    (!value.is_empty()).then(|| value.join(" "))
}

/// The recall line of the log-sieve design's emission (§2.4): one system
/// turn stating the found value, or that none was found.
pub fn recall_line(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("Memory: {value}."),
        None => "Memory: none.".to_owned(),
    }
}

/// Whether a turn takes a recall line: MQAR and relation queries (copy reads
/// its own turn and takes none).
pub fn takes_recall(turn: &Turn2) -> bool {
    matches!(turn.category, Category2::Mqar | Category2::Relation)
}

/// The oracle's recall value for a turn that takes one ([`takes_recall`]):
/// the value the reply must state, or `None` for an abstention.
pub fn oracle_recall(turn: &Turn2) -> Option<&str> {
    if turn.tag.abstain {
        None
    } else {
        turn.tag.answer.as_deref()
    }
}

/// The words of `text`, each with whether a clause ends after it: at
/// punctuation, or at the end of the text.
fn clause_words(text: &str) -> Vec<(String, bool)> {
    let mut all = Vec::new();
    for clause in
        text.split(|c: char| matches!(c, ',' | ';' | ':' | '.' | '!' | '?' | '(' | ')' | '\n'))
    {
        let clause = words(clause);
        let last = clause.len().saturating_sub(1);
        for (i, word) in clause.into_iter().enumerate() {
            all.push((word, i == last));
        }
    }
    all
}

/// [`Rule::Nlet`]. The history is every earlier user turn and reply, in
/// order, as one word sequence; a two-word occurrence may not be the query's
/// own text.
fn nlet_reply(history: &[Turn2], query: &Turn2) -> String {
    let tail = words(&query.user);
    if tail.len() < 2 {
        return DONT_KNOW.to_owned();
    }
    let tail = &tail[tail.len() - 2..];
    let mut flat: Vec<(String, bool)> = Vec::new();
    for turn in history {
        flat.extend(clause_words(&turn.user));
        flat.extend(clause_words(&turn.reply));
    }
    let Some(at) = (0..flat.len().saturating_sub(1))
        .rev()
        .find(|&i| flat[i].0 == tail[0] && flat[i + 1].0 == tail[1])
    else {
        return DONT_KNOW.to_owned();
    };
    let mut follow: Vec<&str> = Vec::new();
    if !flat[at + 1].1 {
        for (word, end) in &flat[at + 2..] {
            follow.push(word.as_str());
            if *end {
                break;
            }
        }
    }
    follow.join(" ")
}

/// The keys [`Scorecard::a1_gate`] reads: MQAR recall at each distance and
/// open-relation recall.
pub fn gated_keys() -> Vec<String> {
    DISTANCES
        .iter()
        .map(|d| format!("mqar/distance/{d}"))
        .chain(["relation/open/recall".to_owned()])
        .collect()
}

/// Whether `pass` of `of` is below [`FREEZE_LIMIT`] (an empty cell is not).
fn below_limit(pass: usize, of: usize) -> bool {
    of > 0 && pass * FREEZE_LIMIT.1 < of * FREEZE_LIMIT.0
}

impl Scorecard {
    /// Whether every gated key has items and a pass rate below
    /// [`FREEZE_LIMIT`]: what an untrained rule must do for the instrument to
    /// freeze.
    pub fn below_freeze_limit(&self) -> bool {
        gated_keys().iter().all(|key| {
            let (pass, of) = self.cell(key);
            below_limit(pass, of)
        })
    }
}

/// The upper end of the 95% Wilson interval of `pass` in `of`.
pub fn wilson_upper(pass: usize, of: usize) -> f64 {
    if of == 0 {
        return 1.0;
    }
    let n = of as f64;
    let p = pass as f64 / n;
    let z2 = 1.96f64 * 1.96;
    let centre = p + z2 / (2.0 * n);
    let margin = 1.96 * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    ((centre + margin) / (1.0 + z2 / n)).min(1.0)
}

/// One row (a rule, or [`REFERENCE`]) over one cell's episodes.
#[derive(Default)]
pub struct RuleRun {
    pub card: Scorecard,
    /// Pass tallies of each gated key by the query template of the item, for
    /// the leak report.
    pub templates: BTreeMap<String, BTreeMap<String, (usize, usize)>>,
}

/// The gated key a turn is counted under, if it is a gated item.
fn gate_key(turn: &Turn2) -> Option<String> {
    if let Some(mqar) = &turn.tag.mqar {
        return Some(format!("mqar/distance/{}", mqar.target_distance));
    }
    (turn.category == Category2::Relation && turn.tag.pool == Some(Pool::Open) && !turn.tag.abstain)
        .then(|| "relation/open/recall".to_owned())
}

/// Every rule and the reference replies over `conversations` episodes of
/// `cell`: each retrieval turn is answered after the episode's reference
/// history and judged by [`judge_v2`].
pub fn run_rules(
    world: &mut MWorld2<'_>,
    rng: &mut Rng,
    cell: Cell,
    conversations: usize,
) -> Result<BTreeMap<&'static str, RuleRun>> {
    run_rows(world, rng, cell, conversations, &Rule::ALL)
}

/// The retrieval routes ([`Rule::ROUTES`]) and the reference replies over
/// `conversations` episodes of `cell`, exactly as [`run_rules`] scores the
/// freeze rules (same draws for the same world, seed and cell).
pub fn run_route(
    world: &mut MWorld2<'_>,
    rng: &mut Rng,
    cell: Cell,
    conversations: usize,
) -> Result<BTreeMap<&'static str, RuleRun>> {
    run_rows(world, rng, cell, conversations, &Rule::ROUTES)
}

fn run_rows(
    world: &mut MWorld2<'_>,
    rng: &mut Rng,
    cell: Cell,
    conversations: usize,
    rules: &[Rule],
) -> Result<BTreeMap<&'static str, RuleRun>> {
    let mut runs: BTreeMap<&'static str, RuleRun> = BTreeMap::new();
    for rule in rules {
        runs.entry(rule.name()).or_default();
    }
    runs.entry(REFERENCE).or_default();
    for _ in 0..conversations {
        let conversation = world.conversation_in(rng, cell)?;
        for (index, turn) in conversation.turns.iter().enumerate() {
            if !is_retrieval(turn) {
                continue;
            }
            let history = &conversation.turns[..index];
            let mut replies: Vec<(&'static str, String)> = Vec::with_capacity(rules.len() + 1);
            for rule in rules {
                replies.push((rule.name(), rule.reply(history, turn)));
            }
            replies.push((REFERENCE, turn.reply.clone()));
            for (name, reply) in replies {
                let pass = judge_v2(&turn.checks, &turn.user, &reply);
                let run = runs.entry(name).or_default();
                run.card.record(turn, pass);
                if let Some(key) = gate_key(turn) {
                    let template = turn
                        .tag
                        .template
                        .clone()
                        .unwrap_or_else(|| "(none)".to_owned());
                    let tally = run
                        .templates
                        .entry(key)
                        .or_default()
                        .entry(template)
                        .or_default();
                    tally.0 += usize::from(pass);
                    tally.1 += 1;
                }
            }
        }
    }
    Ok(runs)
}

/// The freeze decision over the rule runs of one cell (the development cell
/// decides): whether every rule is below [`FREEZE_LIMIT`] on every gated key,
/// each rule's rate on each, and, for a key that leaks, the query templates
/// behind it.
pub fn freeze_report(runs: &BTreeMap<&'static str, RuleRun>) -> Value {
    let mut ok = true;
    let mut gated: BTreeMap<&str, BTreeMap<String, Value>> = BTreeMap::new();
    let mut leaks = Vec::new();
    for rule in Rule::ALL {
        let run = runs.get(rule.name());
        let mut rows = BTreeMap::new();
        for key in gated_keys() {
            let (pass, of) = run.map_or((0, 0), |run| run.card.cell(&key));
            let below = below_limit(pass, of);
            ok &= below;
            let rate = if of == 0 {
                0.0
            } else {
                pass as f64 / of as f64
            };
            rows.insert(
                key.clone(),
                json!({
                    "pass": pass,
                    "of": of,
                    "rate": rate,
                    "wilson95_upper": wilson_upper(pass, of),
                    "below_limit": below,
                }),
            );
            if !below {
                let mut templates: Vec<(String, usize, usize)> = run
                    .and_then(|run| run.templates.get(&key))
                    .map(|by| {
                        by.iter()
                            .map(|(template, &(pass, of))| (template.clone(), pass, of))
                            .collect()
                    })
                    .unwrap_or_default();
                templates.sort_by(|a, b| (b.1 * a.2).cmp(&(a.1 * b.2)).then_with(|| a.0.cmp(&b.0)));
                let by_template: Vec<Value> = templates
                    .iter()
                    .map(|(template, pass, of)| {
                        json!({
                            "template": template,
                            "pass": pass,
                            "of": of,
                            "rate": if *of == 0 { 0.0 } else { *pass as f64 / *of as f64 },
                        })
                    })
                    .collect();
                leaks.push(json!({
                    "rule": rule.name(),
                    "key": key,
                    "pass": pass,
                    "of": of,
                    "by_template": by_template,
                }));
            }
        }
        gated.insert(rule.name(), rows);
    }
    json!({
        "limit": FREEZE_LIMIT.0 as f64 / FREEZE_LIMIT.1 as f64,
        "rule": "instrument_freeze_ok is true only if both rules are strictly below 0.6 on every gated key (MQAR recall at each distance and open-relation recall, on the development cell) and every gated key has items",
        "instrument_freeze_ok": ok,
        "gated": gated,
        "leaks": leaks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::milestone_world::judge;
    use crate::milestone_world_v2_probe::{probe, static_report, ProbeGroup, ProbeItem, ProbeTurn};

    /// The digest and a rendering fingerprint of v1 as R1 sealed it (the same
    /// values on `origin/main` before v2 existed). v2 must never move them.
    const V1_DIGEST: &str = "5efe0de9c1b4c3b39a03182c82c5c0683e456760d6d6f11ca81a0d0a340e3bd7";
    const V1_RENDERING: &str = "df1a9c20d4140b2851447be9d672a56e727e3cf97352cdb5459b2655029e8b9f";

    /// About one token per three characters.
    fn toy(text: &str) -> usize {
        text.chars().count().div_ceil(3)
    }

    fn world() -> MWorld2<'static> {
        MWorld2::new(&toy, Mix::default()).expect("a valid mix")
    }

    fn only(mqar: f64, copy: f64, relation: f64, other: f64) -> MWorld2<'static> {
        MWorld2::new(
            &toy,
            Mix {
                mqar,
                copy,
                relation,
                other,
                closed: CLOSED_SHARE,
            },
        )
        .expect("a valid mix")
    }

    fn episodes(
        world: &mut MWorld2<'_>,
        split: Split,
        seed: u64,
        count: usize,
    ) -> Vec<Conversation2> {
        let mut rng = Rng::new(seed);
        (0..count)
            .map(|_| world.conversation(&mut rng, split).expect("an episode"))
            .collect()
    }

    #[test]
    fn v1_is_pinned() {
        assert_eq!(MWorld::digest(), V1_DIGEST);
        let mut rng = Rng::new(5);
        let mut all = Vec::new();
        for split in [Split::Train, Split::Development] {
            for _ in 0..400 {
                all.push(MWorld::conversation(&mut rng, split));
            }
        }
        let bytes = serde_json::to_vec(&all).expect("serializable");
        assert_eq!(hex::encode(Sha256::digest(&bytes)), V1_RENDERING);
        // v1's judge keeps the four defects #1516 lists; v2 fixes them beside it.
        let echo = "Out of 3 and 7, which is greater?";
        assert!(judge(
            &[Check::Larger {
                answer: 7,
                other: 3
            }],
            echo,
            echo
        ));
        assert!(!judge(
            &[Check::Larger {
                answer: 7,
                other: 3
            }],
            "q",
            "The bigger number is 7, the smaller is 3."
        ));
        assert!(!judge(
            &[Check::LastLetter('B')],
            "q",
            "It starts with the letter b."
        ));
        assert!(judge(
            &[Check::Numbers(vec![1, 2, 3])],
            "q",
            "Here is one way: 2, 3."
        ));
    }

    #[test]
    fn value_pools_are_open_and_disjoint() {
        let strip = |town: &str| -> String {
            let lower = town.to_lowercase();
            TOWN_SUFFIXES
                .iter()
                .find_map(|s| lower.strip_suffix(s))
                .expect("a suffix")
                .to_owned()
        };
        let draw = |split: Split| {
            let mut rng = Rng::new(17);
            let mut pool: BTreeSet<String> = BTreeSet::new();
            for _ in 0..12_000 {
                let (w, n) = (word(&mut rng, split, 2, 3), name(&mut rng, split));
                let (t, d) = (town(&mut rng, split), digits(&mut rng, split));
                let (w, n, t, d) = (
                    w.expect("a word"),
                    n.expect("a name"),
                    t.expect("a town"),
                    d.expect("digits"),
                );
                // The key decides: whatever the shape, in exactly one split.
                assert_eq!(split_of(&w), split);
                assert_eq!(split_of(&n), split);
                assert_eq!(split_of(&d), split);
                assert_eq!(split_of(&strip(&t)), split);
                assert!(n.chars().next().is_some_and(char::is_uppercase));
                assert!((2..=4).contains(&d.len()) && !d.starts_with('0'));
                pool.extend([w, n, t, d]);
            }
            pool
        };
        let (train, development) = (draw(Split::Train), draw(Split::Development));
        assert!(train.len() >= 5_000, "{}", train.len());
        assert!(development.len() >= 5_000, "{}", development.len());
        // No development string, in any case, appears in the training pool.
        let lower = |set: &BTreeSet<String>| -> BTreeSet<String> {
            set.iter().map(|s| s.to_lowercase()).collect()
        };
        let (train_lower, development_lower) = (lower(&train), lower(&development));
        assert!(
            train_lower.is_disjoint(&development_lower),
            "{:?}",
            train_lower.intersection(&development_lower).next()
        );
        // Sixty first syllables of the two-syllable universe alone are far past 5,000 per split.
        assert_eq!(ONSETS.len() * VOWELS.len() * CODAS.len(), 2496);
        let (t, d) = (
            two_syllable_count(Split::Train, 60),
            two_syllable_count(Split::Development, 60),
        );
        // 60 first syllables of 2,496 already give 150,000 strings, a quarter
        // of them development.
        assert!(t > 90_000 && d > 25_000, "{t} {d}");
        // Values never reuse a fixed word of either world.
        let mut rng = Rng::new(3);
        for _ in 0..2000 {
            let w = word(&mut rng, Split::Development, 2, 3).expect("a word");
            assert!(!reserved().contains(&w), "{w}");
        }
    }

    #[test]
    fn phrasings_are_disjoint() {
        let train: BTreeSet<&str> = MWorld2::templates(Split::Train).into_iter().collect();
        let development = MWorld2::templates(Split::Development);
        assert!(development.len() > 60);
        for template in development {
            assert!(!train.contains(template), "{template:?} is in both splits");
        }
        // The v2 templates are in the lists the check reads.
        assert!(train.contains(COPY.train[0]) && train.contains(MQAR_LEAD.train[0]));
        for p in [&MQAR_LEAD, &MQAR_QUERY, &COPY] {
            for d in p.development {
                assert!(!p.train.contains(d));
            }
        }
        // Rendered user turns of one split never come from the other split's
        // templates: a development MQAR lead-in never appears in a training run.
        let mut world = only(1.0, 0.0, 0.0, 0.0);
        for conversation in episodes(&mut world, Split::Train, 1, 300) {
            let first = &conversation.turns[0].user;
            assert!(
                MQAR_LEAD
                    .development
                    .iter()
                    .all(|t| !first.starts_with(t.split("{p}").next().unwrap_or("?"))),
                "{first}"
            );
        }
    }

    #[test]
    fn every_reference_reply_passes_its_own_checks() {
        let mut world = world();
        for split in [Split::Train, Split::Development] {
            let all = episodes(&mut world, split, 7, 3000);
            let mut kinds: BTreeMap<Kind, usize> = BTreeMap::new();
            for conversation in &all {
                *kinds.entry(conversation.kind).or_default() += 1;
                assert!(!conversation.turns.is_empty());
                assert!(conversation.tokens <= CONTEXT, "{}", conversation.tokens);
                assert_eq!(
                    conversation.tokens,
                    world.meter().document(&conversation.turns)
                );
                for turn in &conversation.turns {
                    assert!(
                        judge_v2(&turn.checks, &turn.user, &turn.reply),
                        "{:?}: {:?} -> {:?} fails {:?}",
                        turn.intent,
                        turn.user,
                        turn.reply,
                        turn.checks
                    );
                    // v1's three defective checks are always the v2 variants.
                    assert!(turn.checks.iter().all(|c| !matches!(
                        c,
                        Check2::V1(Check::Larger { .. } | Check::LastLetter(_) | Check::Numbers(_))
                    )));
                }
            }
            for kind in [Kind::Mqar, Kind::Copy, Kind::Relation, Kind::Other] {
                assert!(
                    kinds.get(&kind).copied().unwrap_or(0) > 200,
                    "{kind:?} {kinds:?}"
                );
            }
        }
    }

    #[test]
    fn a_filler_reply_fails_every_retrieval_item() {
        let mut world = world();
        // The prompt's own words are not a retrieval, and neither is a hedge.
        let mut retrieval = 0;
        for split in [Split::Train, Split::Development] {
            for conversation in episodes(&mut world, split, 19, 2500) {
                for turn in &conversation.turns {
                    let is_retrieval = matches!(turn.category, Category2::Mqar | Category2::Copy)
                        || (turn.category == Category2::Relation && !turn.tag.abstain);
                    if is_retrieval {
                        retrieval += 1;
                        assert!(
                            !judge_v2(&turn.checks, &turn.user, "I'm not sure."),
                            "{:?}: {:?} passes a hedge",
                            turn.intent,
                            turn.checks
                        );
                        assert!(
                            !judge_v2(&turn.checks, &turn.user, &turn.user),
                            "{:?}: passes an echo of the question",
                            turn.intent
                        );
                    }
                    // A long unrelated reply fails every turn of every kind.
                    let filler =
                        "The main difference is a simple yet simple way to spend the day at the park.";
                    assert!(
                        !judge_v2(&turn.checks, &turn.user, filler),
                        "{:?}: {:?} passes filler",
                        turn.intent,
                        turn.checks
                    );
                }
            }
        }
        assert!(retrieval > 3000, "{retrieval}");
        // The stated value of an open relation is the only thing that answers.
        let mut world = only(0.0, 0.0, 1.0, 0.0);
        let mut checked = 0;
        for conversation in episodes(&mut world, Split::Development, 23, 800) {
            let query = conversation.turns.last().expect("a query");
            if query.tag.abstain {
                assert!(judge_v2(&query.checks, &query.user, "I'm not sure."));
                continue;
            }
            checked += 1;
            let stated = conversation.turns[0]
                .checks
                .iter()
                .find_map(|c| match c {
                    Check2::V1(Check::AnyOf(list)) => list.last().cloned(),
                    _ => None,
                })
                .expect("the stated value");
            // The first stated value does not answer after an update; the
            // reference reply does.
            assert!(judge_v2(&query.checks, &query.user, &query.reply));
            if conversation
                .turns
                .iter()
                .any(|t| t.intent.ends_with("_update"))
            {
                let stale = format!("It is {stated}.");
                assert!(!judge_v2(&query.checks, &query.user, &stale), "{stale}");
            }
        }
        assert!(checked > 400);
    }

    #[test]
    fn mqar_distances_fall_within_their_buckets() {
        let mut world = only(1.0, 0.0, 0.0, 0.0);
        let mut cells: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        let mut reduced: BTreeMap<usize, usize> = BTreeMap::new();
        let mut filler_turns: BTreeMap<usize, usize> = BTreeMap::new();
        for split in [Split::Train, Split::Development] {
            for conversation in episodes(&mut world, split, 29, 720) {
                assert_eq!(conversation.kind, Kind::Mqar);
                assert!(conversation.tokens <= CONTEXT, "{}", conversation.tokens);
                let query = conversation.turns.last().expect("a query");
                let mqar = query.tag.mqar.as_ref().expect("an MQAR tag");
                assert_eq!(query.category, Category2::Mqar);
                assert!(DISTANCES.contains(&mqar.target_distance));
                assert!(PAIR_COUNTS.contains(&mqar.n) && mqar.n <= mqar.n_requested);
                assert!(
                    mqar.distance >= mqar.target_distance
                        && mqar.distance <= mqar.target_distance + tolerance(mqar.target_distance),
                    "{mqar:?}"
                );
                // The recorded distance is what the meter measures on the turns.
                assert_eq!(mqar.distance, world.meter().distance(&conversation.turns));
                assert_eq!(mqar.tokens, conversation.tokens);
                assert!(mqar.pair_distance >= mqar.distance && mqar.queried < mqar.n);
                *cells
                    .entry((mqar.target_distance, mqar.n_requested))
                    .or_default() += 1;
                if mqar.n < mqar.n_requested {
                    *reduced.entry(mqar.target_distance).or_default() += 1;
                }
                *filler_turns.entry(mqar.target_distance).or_default() +=
                    conversation.turns.len() - 2;
                // The assertion states exactly n pairs and the queried one is
                // among them; the reply carries its value and no other value.
                let assertion = &conversation.turns[0].user;
                let (_, stated) = assertion.rsplit_once(": ").expect("a lead-in");
                assert_eq!(stated.matches(" is ").count(), mqar.n, "{assertion}");
                let value = query
                    .checks
                    .iter()
                    .find_map(|c| match c {
                        Check2::V1(Check::AnyOf(v)) => v.first().cloned(),
                        _ => None,
                    })
                    .expect("the value");
                assert!(assertion.to_lowercase().contains(&format!(" is {value}")));
                assert!(query.reply.to_lowercase().contains(&value));
            }
        }
        // Every (D, N-requested) cell is drawn, evenly.
        assert_eq!(cells.len(), 9, "{cells:?}");
        assert!(cells.values().all(|&n| n == 160), "{cells:?}");
        // Nothing needs reducing at 16 and 64; 200 tokens cannot hold eight
        // pairs plus the episode in a 256-token context, so those items ask for
        // fewer pairs and say so.
        assert_eq!(reduced.get(&16).copied().unwrap_or(0), 0);
        assert!(reduced.get(&200).copied().unwrap_or(0) > 100, "{reduced:?}");
        // Longer distances use more filler.
        assert!(filler_turns[&16] < filler_turns[&64] && filler_turns[&64] < filler_turns[&200]);
    }

    #[test]
    fn copy_is_an_exact_word_sequence() {
        let mut world = only(0.0, 1.0, 0.0, 0.0);
        for conversation in episodes(&mut world, Split::Development, 31, 400) {
            let turn = &conversation.turns[0];
            let target = match &turn.checks[0] {
                Check2::CopyExact(words) => words.clone(),
                other => panic!("{other:?}"),
            };
            assert!((3..=8).contains(&target.len()));
            assert!(target.iter().all(|w| split_of(w) == Split::Development));
            assert_eq!(words(&turn.reply), target);
            // Case and punctuation are ignored; nothing else is.
            let shouted = target.join(" ").to_uppercase();
            assert!(judge_v2(&turn.checks, &turn.user, &shouted));
            assert!(judge_v2(
                &turn.checks,
                &turn.user,
                &format!("{}.", target.join(", "))
            ));
            let mut fewer = target.clone();
            fewer.pop();
            assert!(!judge_v2(&turn.checks, &turn.user, &fewer.join(" ")));
            let mut swapped = target.clone();
            swapped.swap(0, 1);
            assert!(!judge_v2(&turn.checks, &turn.user, &swapped.join(" ")));
            let extra = format!("Sure: {}", target.join(" "));
            assert!(!judge_v2(&turn.checks, &turn.user, &extra));
            assert!(!judge_v2(
                &turn.checks,
                &turn.user,
                &format!("{} {}", target.join(" "), target[0])
            ));
        }
    }

    #[test]
    fn the_1516_regressions() {
        let larger = |answer, other, user: &str, reply: &str| {
            judge_v2(&[Check2::Larger { answer, other }], user, reply)
        };
        // 1. An echo of the prompt that states both numbers is not an answer,
        // with or without the comma, and a real answer still is one.
        let user = "Out of 3 and 7, which is greater?";
        assert!(!larger(7, 3, user, "Out of 3 and 7, which is greater?"));
        assert!(!larger(7, 3, user, "Out of 3 and 7 which is greater"));
        assert!(!larger(7, 3, user, "Which is greater, 3 or 7?"));
        assert!(!larger(
            7,
            3,
            "Pick the larger number: 3 or 7.",
            "Pick the larger number: 3 or 7."
        ));
        assert!(larger(7, 3, user, "Out of 3 and 7, 7 is greater."));
        assert!(larger(7, 3, user, "7 is greater."));
        assert!(larger(7, 3, user, "7."));
        // 2. The negation window ends with the clause.
        let q = "Which is bigger, 3 or 7?";
        assert!(larger(7, 3, q, "The bigger number is 7, the smaller is 3."));
        assert!(larger(
            7,
            3,
            q,
            "The bigger number is 7 and the smaller number is 3."
        ));
        assert!(larger(7, 3, q, "7 is bigger, 3 is smaller."));
        assert!(larger(7, 3, q, "7 is bigger than 3."));
        assert!(larger(12, 3, q, "The bigger one is 12."));
        assert!(!larger(7, 3, q, "3 is bigger than 7."));
        assert!(!larger(7, 3, q, "7 is smaller."));
        assert!(!larger(
            7,
            3,
            q,
            "The bigger number is 3, the smaller is 7."
        ));
        assert!(!larger(7, 3, q, "7 is bigger. No, 3 is bigger."));
        assert!(!larger(7, 3, q, "3 and 7."));
        // 3. LastLetter is case-insensitive; the article and the pronoun are
        // still not letters.
        let letter = |c, reply| judge_v2(&[Check2::LastLetter(c)], "q", reply);
        assert!(letter('B', "banana starts with the letter B."));
        assert!(letter('B', "banana starts with the letter b."));
        assert!(letter('B', "It starts with b."));
        assert!(letter('B', "b"));
        assert!(letter('A', "apple starts with the letter a."));
        assert!(letter('A', "apple starts with the letter A."));
        assert!(letter('A', "a"));
        assert!(!letter('A', "I'd recommend using a mixture to create a"));
        assert!(!letter('A', "A gentle breeze is here."));
        assert!(!letter('A', "It starts with a banana."));
        assert!(!letter('I', "I think it is great."));
        assert!(!letter('I', "and i think so"));
        assert!(letter('I', "ice starts with the letter i."));
        assert!(letter('I', "I think it starts with I."));
        assert!(letter(
            'Z',
            "Zebra starts with the letter Z. I hope that helps!"
        ));
        assert!(!letter('B', "It starts with the letter c."));
        // 4. The pronoun "one" is not the number 1.
        let numbers = |reply| judge_v2(&[Check2::Numbers(vec![1, 2, 3])], "q", reply);
        assert!(!numbers("Here is one way: 2, 3."));
        assert!(!numbers("Pick which one, then 2 and 3."));
        assert!(!numbers("Here is one: 2, 3."));
        assert!(!numbers("The first one: 2, 3."));
        assert!(numbers("One, two, three."));
        assert!(numbers("1, 2, 3."));
        assert!(numbers("1 2 3"));
        assert!(numbers("Two, three, one."));
        assert!(!numbers("Two and three."));
        let count = [
            Check2::Numbers(vec![2, 3, 4]),
            Check2::V1(Check::NoNumber(5)),
        ];
        assert!(judge_v2(&count, "q", "2, 3, 4."));
        assert!(!judge_v2(&count, "q", "2, 3, 4, 5."));
    }

    #[test]
    fn the_digest_and_the_streams_are_stable() {
        let digest = MWorld2::digest();
        assert_eq!(digest.len(), 64);
        assert_eq!(digest, MWorld2::digest());
        assert_ne!(digest, MWorld::digest());
        // Two constructions draw the same stream from the same seed.
        let first = episodes(&mut world(), Split::Development, 41, 400);
        let second = episodes(&mut world(), Split::Development, 41, 400);
        assert_eq!(first, second);
        let other_seed = episodes(&mut world(), Split::Development, 42, 400);
        assert_ne!(first, other_seed);
        // The split changes the stream and the values.
        let train = episodes(&mut world(), Split::Train, 41, 400);
        assert_ne!(first, train);
        // Tags and checks survive serialization.
        let text = serde_json::to_string(&first).expect("serializable");
        let back: Vec<Conversation2> = serde_json::from_str(&text).expect("deserializable");
        assert_eq!(first, back);
        // Bad mixes are refused.
        for (a, b, c, d) in [
            (0.5, 0.5, 0.5, 0.0),
            (-0.1, 0.6, 0.3, 0.2),
            (0.2, 0.2, 0.2, 0.2),
        ] {
            let mix = Mix {
                mqar: a,
                copy: b,
                relation: c,
                other: d,
                closed: 0.2,
            };
            assert!(MWorld2::new(&toy, mix).is_err(), "{mix:?}");
        }
    }

    #[test]
    fn the_mix_pools_and_freshness_are_as_designed() {
        let mut world = world();
        let all = episodes(&mut world, Split::Development, 53, 4000);
        let share = |kind: Kind| all.iter().filter(|c| c.kind == kind).count() as f64 / 4000.0;
        assert!(
            (share(Kind::Mqar) - 0.35).abs() < 0.03,
            "{}",
            share(Kind::Mqar)
        );
        assert!(
            (share(Kind::Copy) - 0.10).abs() < 0.03,
            "{}",
            share(Kind::Copy)
        );
        assert!(
            (share(Kind::Relation) - 0.30).abs() < 0.03,
            "{}",
            share(Kind::Relation)
        );
        assert!(
            (share(Kind::Other) - 0.25).abs() < 0.03,
            "{}",
            share(Kind::Other)
        );
        // Closed relations are the minority of relation episodes and are tagged.
        let (mut open, mut closed) = (0, 0);
        let mut users = BTreeSet::new();
        for conversation in all.iter().filter(|c| c.kind == Kind::Relation) {
            let query = conversation.turns.last().expect("a query");
            let pool = query.tag.pool.expect("a pool tag");
            let closed_names = ["job", "home", "favorite_food", "favorite_color"];
            let relation_of = |intent: &str| intent.rsplit_once('_').map(|(n, _)| n.to_owned());
            let asked_closed =
                relation_of(&query.intent).is_some_and(|n| closed_names.contains(&n.as_str()));
            assert_eq!(pool == Pool::Closed, asked_closed, "{}", query.intent);
            let stated = &conversation.turns[0].intent;
            if relation_of(stated).is_some_and(|n| closed_names.contains(&n.as_str())) {
                closed += 1;
            } else {
                open += 1;
                users.insert(conversation.turns[0].user.clone());
            }
        }
        // About four open episodes to one closed (0.8 to 0.2); three to one is
        // over four standard deviations from that.
        assert!(closed > 100 && open > 3 * closed, "{open} {closed}");
        // Open values are fresh each episode: almost no assertion repeats.
        assert!(
            users.len() as f64 > 0.95 * open as f64,
            "{} {open}",
            users.len()
        );
        // Fresh MQAR keys and values too.
        let mut keys = BTreeSet::new();
        let mut asserted = 0;
        for conversation in all.iter().filter(|c| c.kind == Kind::Mqar) {
            keys.insert(conversation.turns[0].user.clone());
            asserted += 1;
        }
        assert_eq!(keys.len(), asserted);
    }

    #[test]
    fn the_a1_gate_is_exact() {
        let query = |distance: usize, pass_open: Option<bool>| {
            let mqar = Turn2 {
                intent: "mqar_query".into(),
                category: Category2::Mqar,
                user: String::new(),
                reply: String::new(),
                checks: Vec::new(),
                tag: Tag {
                    pool: Some(Pool::Open),
                    abstain: false,
                    mqar: Some(Mqar {
                        n: 2,
                        n_requested: 2,
                        target_distance: distance,
                        distance,
                        pair_distance: distance,
                        queried: 0,
                        tokens: 100,
                    }),
                    ..Tag::default()
                },
            };
            let relation = pass_open.map(|_| Turn2 {
                intent: "user_name_query".into(),
                category: Category2::Relation,
                user: String::new(),
                reply: String::new(),
                checks: Vec::new(),
                tag: Tag {
                    pool: Some(Pool::Open),
                    abstain: false,
                    mqar: None,
                    ..Tag::default()
                },
            });
            (mqar, relation)
        };
        // Ten items per distance and for the open relation; `fails` misses.
        let card = |fails: [usize; 4], abstain_fails: usize| {
            let mut card = Scorecard::default();
            for (i, d) in DISTANCES.iter().enumerate() {
                for k in 0..10 {
                    card.record(&query(*d, None).0, k >= fails[i]);
                }
            }
            for k in 0..10 {
                let turn = query(16, Some(true)).1.expect("relation");
                card.record(&turn, k >= fails[3]);
            }
            for k in 0..10 {
                let mut turn = query(16, Some(true)).1.expect("relation");
                turn.tag.abstain = true;
                card.record(&turn, k >= abstain_fails);
            }
            card
        };
        assert!(card([0, 0, 0, 0], 0).a1_gate());
        // 9 of 10 is exactly 0.9 and passes; 8 of 10 fails, at any distance.
        assert!(card([1, 1, 1, 1], 0).a1_gate());
        assert!(!card([2, 0, 0, 0], 0).a1_gate());
        assert!(!card([0, 2, 0, 0], 0).a1_gate());
        assert!(!card([0, 0, 2, 0], 0).a1_gate());
        assert!(!card([0, 0, 0, 2], 0).a1_gate());
        // Abstentions are reported apart and do not decide the gate.
        assert!(card([0, 0, 0, 0], 10).a1_gate());
        // A distance with no items fails; so does an empty scorecard.
        let mut missing = Scorecard::default();
        for d in [16, 64] {
            for _ in 0..10 {
                missing.record(&query(d, None).0, true);
            }
        }
        for _ in 0..10 {
            missing.record(&query(16, Some(true)).1.expect("relation"), true);
        }
        assert!(!missing.a1_gate());
        assert!(!Scorecard::default().a1_gate());
        let report = card([1, 1, 1, 1], 5).to_json();
        assert_eq!(report["a1_gate"], json!(true));
        assert_eq!(report["mqar"]["by_distance"]["64"]["of"], json!(10));
        assert_eq!(report["relation"]["open"]["pass"], json!(9));
        assert_eq!(report["relation"]["open_abstain"]["pass"], json!(5));
        assert_eq!(report["relation"]["closed"]["of"], json!(0));
    }

    // -- The council's A1 amendments (revision 2.1) --------------------------

    /// The digest of revision 2.1, the world the amendments freeze, and of
    /// revision 2.0 as first written (c5b84173), which it supersedes. Both were
    /// computed offline: the canonical JSON `digest()` hashes was rebuilt from
    /// the two source files with jq (sorted keys, compact) and hashed with
    /// `shasum -a 256`; the same procedure reproduces v1's pinned digest
    /// exactly. If this assertion fails, the tables or constants moved: read the
    /// value it prints, re-derive it, and post the new digest on issue 1511
    /// before any treatment run. It hashes tables and constants, not generator
    /// code: the balanced relation queries of #1541 changed no table, so it
    /// did not move, and `STREAM_DIGEST` pins what the generator draws.
    const V2_DIGEST: &str = "04ad3bb0bf68d4213d90b41ffb886711ffa4c98043e5ac15f58719d7ce68ae4f";
    const V2_DIGEST_2_0: &str = "3a4ee743766492c221747373ffcdd43b249e65d049e28d264869af634d8ffc26";

    #[test]
    fn the_v2_digest_is_pinned() {
        assert_eq!(MWorld2::digest(), V2_DIGEST);
        assert_ne!(V2_DIGEST, V2_DIGEST_2_0);
        assert_eq!(REVISION, "2.1");
        // v1 is untouched by the revision.
        assert_eq!(MWorld::digest(), V1_DIGEST);
    }

    /// The SHA-256 of the fixed-seed revision-2.1 episode stream that
    /// [`stream_digest`] draws. It was taken from the first build that compiled
    /// this test: GitHub run 36730688569 at `f4b0c5da` (#1541). Any later change
    /// of this constant is a change of the instrument: say why on issue 1511
    /// before any treatment run.
    ///
    /// [`MWorld2::digest`] hashes the tables and constants only, so it does
    /// not move with the relation generator; this digest is what does.
    const STREAM_DIGEST: &str = "0167c1a06d0df94b6e7ee8662298099190b94ab5186bdcba81842b1466dc70ea";

    /// SHA-256 over the JSON of a fixed-seed stream of revision-2.1 episodes
    /// under the toy meter: for each of the four cells, 150 episodes of the
    /// default mix (seed 2101) and 150 relation-only episodes (seed 2102).
    fn stream_digest() -> String {
        let mut hasher = Sha256::new();
        for cell in Cell::ALL {
            for (mut generator, seed) in [(world(), 2_101u64), (only(0.0, 0.0, 1.0, 0.0), 2_102)] {
                let mut rng = Rng::new(seed);
                for _ in 0..150 {
                    let conversation = generator
                        .conversation_in(&mut rng, cell)
                        .expect("an episode");
                    hasher.update(serde_json::to_vec(&conversation).expect("serializable"));
                    hasher.update([0u8]);
                }
            }
        }
        hex::encode(hasher.finalize())
    }

    #[test]
    fn the_revision_2_1_episode_stream_is_pinned() {
        let digest = stream_digest();
        // The stream is a function of its seeds alone.
        assert_eq!(digest, stream_digest());
        assert_ne!(
            STREAM_DIGEST, "PENDING",
            "PENDING: the revision-2.1 episode stream is not pinned yet; its digest is {digest}"
        );
        assert_eq!(
            digest, STREAM_DIGEST,
            "the revision-2.1 episode stream moved: a generator or a table changed"
        );
    }

    /// SHA-256 over the JSON of the D19 session draw's shape under the toy
    /// meter: development split, default mix, seed 9101, 300 conversations
    /// (`m-world session world=v2`'s defaults). Taken on `origin/main`
    /// `e4922ec5`, before world=v2c existed (Step 4, #820): adding v2c must
    /// leave every v2 draw byte-identical.
    const V2_SEED_9101_DIGEST: &str =
        "e3cb5d2a65b856b6db4432f5e1d031487ee742904b195a166c6b1ccf0bf8410f";

    fn session_draw_digest(world: &mut MWorld2<'_>) -> String {
        let mut hasher = Sha256::new();
        let mut rng = Rng::new(9_101);
        for _ in 0..300 {
            let conversation = world
                .conversation(&mut rng, Split::Development)
                .expect("an episode");
            hasher.update(serde_json::to_vec(&conversation).expect("serializable"));
            hasher.update([0u8]);
        }
        hex::encode(hasher.finalize())
    }

    #[test]
    fn the_v2_seed_9101_draw_is_unchanged() {
        let digest = session_draw_digest(&mut world());
        assert_eq!(
            digest, V2_SEED_9101_DIGEST,
            "the world=v2 seed-9101 draw moved"
        );
        // Naming the variant explicitly draws the same stream.
        let mut explicit = world().with_variant(Variant::V2);
        assert_eq!(session_draw_digest(&mut explicit), V2_SEED_9101_DIGEST);
        assert_eq!(MWorld2::digest_for(Variant::V2), MWorld2::digest());
        assert_eq!(MWorld2::digest(), V2_DIGEST);
    }

    #[test]
    fn v2c_rehearses_the_key_in_its_asserted_casing() {
        let mut v2c = world().with_variant(Variant::V2c);
        assert_eq!(v2c.variant(), Variant::V2c);
        assert_ne!(session_draw_digest(&mut v2c), V2_SEED_9101_DIGEST);
        assert_ne!(MWorld2::digest_for(Variant::V2c), MWorld2::digest());
        let mut v2c = world().with_variant(Variant::V2c);
        let mut v2 = world();
        let (mut rng_c, mut rng_2) = (Rng::new(9_101), Rng::new(9_101));
        let (mut queries, mut rehearsals, mut same_choice, mut paired) = (0, 0, 0, 0);
        for _ in 0..300 {
            let c = v2c
                .conversation(&mut rng_c, Split::Development)
                .expect("a v2c episode");
            let b = v2
                .conversation(&mut rng_2, Split::Development)
                .expect("a v2 episode");
            let users =
                |x: &Conversation2| x.turns.iter().map(|t| t.user.clone()).collect::<Vec<_>>();
            let same_episode = users(&c) == users(&b);
            paired += usize::from(same_episode);
            let Some(assertion) = c.turns.iter().find(|t| t.intent == "mqar_assert") else {
                continue;
            };
            let query = c.turns.last().expect("a query");
            assert_eq!(query.intent, "mqar_query");
            assert!(judge_v2(&query.checks, &query.user, &query.reply));
            queries += 1;
            let value = query.tag.answer.as_deref().expect("an answer");
            if let Some(rest) = query.reply.strip_prefix("So ") {
                rehearsals += 1;
                let key = rest
                    .strip_suffix(&format!(" is {value}."))
                    .expect("So {k} is {v}.");
                // The key is written exactly as asserted and as asked.
                assert!(assertion.user.contains(&format!(" {key} is {value}")));
                assert!(query.user.contains(&format!(" {key}")));
                assert_eq!(key, key.to_lowercase());
                if same_episode {
                    let old = &b.turns.last().expect("a query").reply;
                    assert_eq!(
                        *old,
                        format!("{}{} is {value}.", key[..1].to_uppercase(), &key[1..])
                    );
                    same_choice += 1;
                }
            } else {
                assert!(
                    query.reply.starts_with("It's ") || query.reply.starts_with("That's "),
                    "{:?}",
                    query.reply
                );
                if same_episode {
                    assert_eq!(b.turns.last().expect("a query").reply, query.reply);
                }
            }
        }
        // About a third of the queries rehearse, as in v2 (#1709: 41 of 109);
        // the streams stay paired, episode for episode, until the first fit
        // the one-token frame changes (near the context at D200); after that
        // they are independent draws of the same distribution.
        assert!(queries > 80, "{queries}");
        assert!(
            rehearsals * 5 > queries && rehearsals * 2 < queries,
            "{rehearsals}/{queries}"
        );
        assert!(same_choice > 0 && paired > 0, "{same_choice} {paired}");
    }

    #[test]
    fn variants_parse_and_name_their_worlds() {
        for variant in [Variant::V2, Variant::V2c] {
            assert_eq!(Variant::parse(variant.key()).expect("parses"), variant);
            assert_eq!(
                Variant::from_world_name(Some(variant.world_name())).expect("named"),
                variant
            );
        }
        assert_eq!(Variant::from_world_name(None).expect("v2"), Variant::V2);
        assert!(Variant::parse("v3").is_err());
        assert!(Variant::from_world_name(Some("m-world-v1")).is_err());
    }

    /// A hand-built turn; `values` are the open values its user text states.
    fn hand_turn(user: &str, reply: &str, values: &[&str]) -> Turn2 {
        Turn2 {
            intent: "hand_built".into(),
            category: Category2::Responsive,
            user: user.into(),
            reply: reply.into(),
            checks: Vec::new(),
            tag: Tag {
                values: values.iter().map(|v| (*v).to_owned()).collect(),
                ..Tag::default()
            },
        }
    }

    /// A scorecard holding exactly `pass` of `of` for each key.
    fn card_of(rows: &[(&str, usize, usize)]) -> Scorecard {
        let mut card = Scorecard::default();
        for (key, pass, of) in rows {
            for i in 0..*of {
                card.add((*key).to_owned(), i < *pass);
            }
        }
        card
    }

    fn mqar_item(distance: usize) -> Turn2 {
        Turn2 {
            intent: "mqar_query".into(),
            category: Category2::Mqar,
            user: String::new(),
            reply: String::new(),
            checks: Vec::new(),
            tag: Tag {
                pool: Some(Pool::Open),
                mqar: Some(Mqar {
                    n: 2,
                    n_requested: 2,
                    target_distance: distance,
                    distance,
                    pair_distance: distance,
                    queried: 0,
                    tokens: 100,
                }),
                ..Tag::default()
            },
        }
    }

    fn recall_item(pool: Pool) -> Turn2 {
        Turn2 {
            intent: "user_name_query".into(),
            category: Category2::Relation,
            user: String::new(),
            reply: String::new(),
            checks: Vec::new(),
            tag: Tag {
                pool: Some(pool),
                ..Tag::default()
            },
        }
    }

    #[test]
    fn the_sieve_route_binds_by_exact_identity_and_the_latest_version() {
        let history = [
            hand_turn(
                "Please remember: bol is 47, tamir is kavu.",
                // A reply is the model's own text, never a logged fact.
                "Sure, bol is 99.",
                &["47", "kavu"],
            ),
            hand_turn("The weather is nice.", "It is.", &[]),
        ];
        let ask = |text: &str| hand_turn(text, "", &[]);
        // The query's content atom admits its clause; the value follows it.
        assert_eq!(
            sieve_value(&history, &ask("What is bol?")).as_deref(),
            Some("47")
        );
        assert_eq!(
            sieve_value(&history, &ask("Say what tamir was.")).as_deref(),
            Some("kavu")
        );
        assert_eq!(
            Rule::Sieve.reply(&history, &ask("What is bol?")),
            "It's 47."
        );
        // Unlike R-recency, the asked key decides, not the latest value.
        assert_eq!(Rule::Recency.reply(&history, &ask("What is bol?")), "kavu");
        // The latest version of a key wins.
        let later = [
            history[0].clone(),
            hand_turn("Correction: bol is 52.", "Noted.", &["52"]),
        ];
        assert_eq!(
            sieve_value(&later, &ask("What is bol?")).as_deref(),
            Some("52")
        );
        // No content atom, or no clause sharing one: nothing is found.
        assert_eq!(sieve_value(&history, &ask("What is my name?")), None);
        assert_eq!(sieve_value(&history, &ask("What is zorpleem?")), None);
        assert_eq!(sieve_value(&[], &ask("What is bol?")), None);
        assert_eq!(Rule::Sieve.reply(&[], &ask("What is bol?")), DONT_KNOW);
        // The route is not a freeze rule.
        assert!(!Rule::ALL.contains(&Rule::Sieve));
        assert_eq!(Rule::ROUTES, [Rule::Sieve]);
        // Recall lines.
        assert_eq!(recall_line(Some("47")), "Memory: 47.");
        assert_eq!(recall_line(None), "Memory: none.");
        let mut query = recall_item(Pool::Open);
        query.tag.answer = Some("Zelpur".into());
        assert!(takes_recall(&query));
        assert_eq!(oracle_recall(&query), Some("Zelpur"));
        query.tag.abstain = true;
        assert_eq!(oracle_recall(&query), None);
    }

    #[test]
    fn the_rule_baselines_on_hand_built_episodes() {
        let history = [
            hand_turn(
                "Please remember: bol is 47, tamir is kavu.",
                "Noted.",
                &["47", "kavu"],
            ),
            hand_turn("The weather is nice.", "It is.", &[]),
        ];
        let ask = |text: &str| hand_turn(text, "", &[]);
        // R-recency: the latest recorded value, whichever pair is asked.
        assert_eq!(Rule::Recency.reply(&history, &ask("What is bol?")), "kavu");
        assert_eq!(
            Rule::Recency.reply(&history[1..], &ask("What is bol?")),
            DONT_KNOW
        );
        assert_eq!(Rule::Recency.reply(&[], &ask("What is bol?")), DONT_KNOW);
        // A later statement wins over an earlier one.
        let later = [
            history[0].clone(),
            hand_turn("Actually, my name is Zeta.", "Got it.", &["Zeta"]),
            hand_turn("How are you?", "Fine.", &[]),
        ];
        assert_eq!(
            Rule::Recency.reply(&later, &ask("What is my name?")),
            "Zeta"
        );
        // R-nlet: what follows the latest earlier occurrence of the query's
        // last two words, up to the clause end.
        assert_eq!(
            Rule::Nlet.reply(&history, &ask("Tell me what tamir is.")),
            "kavu"
        );
        assert_eq!(Rule::Nlet.reply(&history, &ask("SAY WHAT BOL IS!")), "47");
        // The last two words decide: "is bol" never occurs in the history.
        assert_eq!(Rule::Nlet.reply(&history, &ask("What is bol?")), DONT_KNOW);
        assert_eq!(
            Rule::Nlet.reply(&history, &ask("Tell me what tamir was.")),
            DONT_KNOW
        );
        // A query of one word, and an empty history, have nothing to match.
        assert_eq!(Rule::Nlet.reply(&history, &ask("Hi")), DONT_KNOW);
        assert_eq!(Rule::Nlet.reply(&[], &ask("What is bol?")), DONT_KNOW);
        // The continuation stops at the clause end, and is empty when the
        // two words end a clause.
        let dog = [hand_turn(
            "My dog goes by Ziggy, and he is tiny.",
            "Got it.",
            &["Ziggy"],
        )];
        assert_eq!(
            Rule::Nlet.reply(&dog, &ask("What did I name my dog?")),
            "goes by ziggy"
        );
        let number = [hand_turn("42 is my lucky number.", "Noted.", &["42"])];
        assert_eq!(
            Rule::Nlet.reply(&number, &ask("Remind me of my lucky number.")),
            ""
        );
        // The latest occurrence wins, and replies are history too.
        let twice = [
            hand_turn("The code word is alpha.", "Okay.", &["alpha"]),
            hand_turn(
                "Hello.",
                "Got it, the code word is beta, and that is all.",
                &[],
            ),
        ];
        assert_eq!(
            Rule::Nlet.reply(&twice, &ask("Remind me of the code word.")),
            "is beta"
        );
        // Judged by the oracle: a recency reply that names the asked value
        // passes, the phrasing that ended in "{k} is" leaked to R-nlet, and
        // the revised phrasing does not.
        let checks = [
            Check2::V1(Check::AnyOf(vec!["kavu".into()])),
            Check2::V1(Check::NoneOf(vec!["47".into()])),
        ];
        let old = "Tell me what tamir is.";
        let revised = "Tell me what tamir was.";
        assert!(judge_v2(
            &checks,
            old,
            &Rule::Recency.reply(&history, &ask(old))
        ));
        assert!(judge_v2(
            &checks,
            old,
            &Rule::Nlet.reply(&history, &ask(old))
        ));
        assert!(!judge_v2(
            &checks,
            revised,
            &Rule::Nlet.reply(&history, &ask(revised))
        ));
    }

    #[test]
    fn no_mqar_query_phrasing_ends_in_the_words_before_the_value() {
        for template in MQAR_QUERY.train.iter().chain(MQAR_QUERY.development) {
            let bare = template.trim_end_matches(['?', '.']);
            assert!(!bare.ends_with("{k} is"), "{template}");
            assert!(bare.contains("{k}"), "{template}");
        }
    }

    #[test]
    fn the_freeze_limit_is_strictly_below_three_fifths() {
        let gated = |pass: usize| {
            card_of(&[
                ("mqar/distance/16", pass, 100),
                ("mqar/distance/64", pass, 100),
                ("mqar/distance/200", pass, 100),
                ("relation/open/recall", pass, 100),
            ])
        };
        assert!(gated(0).below_freeze_limit());
        assert!(gated(59).below_freeze_limit());
        assert!(!gated(60).below_freeze_limit());
        // One leaking key is enough, and a key with no items does not freeze.
        let leaking = card_of(&[
            ("mqar/distance/16", 5, 100),
            ("mqar/distance/64", 5, 100),
            ("mqar/distance/200", 61, 100),
            ("relation/open/recall", 5, 100),
        ]);
        assert!(!leaking.below_freeze_limit());
        let missing = card_of(&[
            ("mqar/distance/16", 5, 100),
            ("mqar/distance/64", 5, 100),
            ("relation/open/recall", 5, 100),
        ]);
        assert!(!missing.below_freeze_limit());
        assert!(!Scorecard::default().below_freeze_limit());
        assert!((wilson_upper(50, 100) - 0.5962).abs() < 2e-3);
        assert!((wilson_upper(0, 0) - 1.0).abs() < 1e-12);
        // The report names the leaking rule and key, and the templates
        // behind it, the most leaking first.
        let mut runs: BTreeMap<&'static str, RuleRun> = BTreeMap::new();
        runs.insert(
            Rule::Recency.name(),
            RuleRun {
                card: gated(10),
                templates: BTreeMap::new(),
            },
        );
        let mut leak = RuleRun {
            card: leaking,
            templates: BTreeMap::new(),
        };
        let by_template = leak
            .templates
            .entry("mqar/distance/200".into())
            .or_default();
        by_template.insert("What is {k}?".into(), (11, 50));
        by_template.insert("Say what {k} is.".into(), (50, 50));
        runs.insert(Rule::Nlet.name(), leak);
        let report = freeze_report(&runs);
        assert_eq!(report["instrument_freeze_ok"], json!(false));
        assert_eq!(report["leaks"].as_array().map(Vec::len), Some(1));
        assert_eq!(report["leaks"][0]["rule"], json!("R-nlet"));
        assert_eq!(report["leaks"][0]["key"], json!("mqar/distance/200"));
        assert_eq!(
            report["leaks"][0]["by_template"][0]["template"],
            json!("Say what {k} is.")
        );
        assert_eq!(
            report["gated"]["R-nlet"]["mqar/distance/200"]["below_limit"],
            json!(false)
        );
        assert_eq!(
            report["gated"]["R-recency"]["mqar/distance/200"]["below_limit"],
            json!(true)
        );
        // Both below the limit: the instrument freezes.
        runs.insert(
            Rule::Nlet.name(),
            RuleRun {
                card: gated(3),
                templates: BTreeMap::new(),
            },
        );
        let report = freeze_report(&runs);
        assert_eq!(report["instrument_freeze_ok"], json!(true));
        assert_eq!(report["leaks"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn the_rules_stay_below_the_freeze_limit_on_the_development_cell() {
        let mut world = world();
        let mut rng = Rng::new(9_101);
        let runs = run_rules(&mut world, &mut rng, Cell::GATED, 1_500).expect("rule runs");
        let report = freeze_report(&runs);
        assert_eq!(
            report["instrument_freeze_ok"],
            json!(true),
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
        // The world's own reference replies pass every item: the oracle
        // judges what it should, so the rules' rates are not a harness slip.
        let reference = runs.get(REFERENCE).expect("the reference row");
        assert!(reference.card.a1_gate());
        for key in gated_keys() {
            assert!((reference.card.rate(&key) - 1.0).abs() < 1e-12, "{key}");
        }
        assert!(reference.card.rate("copy") > 0.999);
        // Every gated key was scored.
        for name in [Rule::Recency.name(), Rule::Nlet.name()] {
            for key in gated_keys() {
                assert!(runs[name].card.cell(&key).1 > 50, "{name} {key}");
            }
        }
    }

    /// The (relation, value) every assertion and update of `turns` states, in
    /// reading order. The value is the last word the turn's acknowledgment
    /// check accepts, which is the stated value lowercased.
    fn statements(turns: &[Turn2]) -> Vec<(String, String)> {
        turns
            .iter()
            .filter_map(|turn| {
                let relation = turn
                    .intent
                    .strip_suffix("_assert")
                    .or_else(|| turn.intent.strip_suffix("_update"))?;
                let value = turn.checks.iter().find_map(|check| match check {
                    Check2::V1(Check::AnyOf(list)) => list.last().cloned(),
                    _ => None,
                })?;
                Some((relation.to_owned(), value))
            })
            .collect()
    }

    fn is_closed_relation(name: &str) -> bool {
        relations()
            .iter()
            .any(|r| r.name == name && r.pool() == Pool::Closed)
    }

    #[test]
    fn a_relation_episode_states_a_companion_and_asks_either_relation_evenly() {
        let mut world = only(0.0, 0.0, 1.0, 0.0);
        let all = episodes(&mut world, Split::Development, 61, 2_400);
        let (mut recall, mut companion_asked) = (0usize, 0usize);
        let (mut abstentions, mut closed) = (0usize, 0usize);
        // Episodes that update the queried relation: how many state the
        // companion last (index 0) or the update last (index 1), and under each
        // order (recall items, items that ask the companion).
        let mut order = [0usize; 2];
        let mut asked_by_order = [(0usize, 0usize); 2];
        for conversation in &all {
            let stated = statements(&conversation.turns);
            let queried = stated
                .first()
                .map(|(name, _)| name.clone())
                .expect("the queried relation is stated first");
            let companion = stated
                .iter()
                .map(|(name, _)| name.as_str())
                .find(|name| *name != queried.as_str())
                .expect("a companion relation is stated");
            // Two relations: the companion stated once and never updated, of
            // the queried relation's pool.
            let in_pair = |name: &str| name == queried.as_str() || name == companion;
            assert!(
                stated.iter().all(|(name, _)| in_pair(name.as_str())),
                "{:?}",
                conversation.turns
            );
            let stated_of = |name: &str, suffix: &str| {
                conversation
                    .turns
                    .iter()
                    .filter(|t| t.intent == format!("{name}_{suffix}"))
                    .count()
            };
            assert_eq!(stated_of(companion, "assert"), 1);
            assert_eq!(stated_of(companion, "update"), 0);
            assert_eq!(stated_of(&queried, "assert"), 1);
            assert!(stated_of(&queried, "update") <= 1);
            assert_eq!(is_closed_relation(&queried), is_closed_relation(companion));
            closed += usize::from(is_closed_relation(&queried));
            let updated = stated_of(&queried, "update") == 1;
            let order_index = if updated {
                let last = stated.last().map(|(name, _)| name.as_str());
                let index = usize::from(last != Some(companion));
                order[index] += 1;
                Some(index)
            } else {
                None
            };
            let query = conversation.turns.last().expect("a query");
            if query.tag.abstain {
                // An abstention asks for a relation stated nowhere.
                abstentions += 1;
                let asked = query
                    .intent
                    .strip_suffix("_absent")
                    .expect("an abstention intent");
                assert!(asked != queried.as_str() && asked != companion, "{asked}");
                continue;
            }
            recall += 1;
            let asked = query.intent.strip_suffix("_query").expect("a query intent");
            assert!(asked == queried.as_str() || asked == companion, "{asked}");
            let asks_companion = asked == companion;
            companion_asked += usize::from(asks_companion);
            if let Some(index) = order_index {
                asked_by_order[index].0 += 1;
                asked_by_order[index].1 += usize::from(asks_companion);
            }
            // The answer is the asked relation's current value, and the checks
            // accept that value alone: not the other relation's, not the
            // asked relation's replaced one.
            let answer = stated
                .iter()
                .rev()
                .find(|(name, _)| name.as_str() == asked)
                .map(|(_, value)| value.clone())
                .expect("the asked relation is stated");
            assert_eq!(
                query.tag.answer.as_ref().map(|a| a.to_lowercase()),
                Some(answer.clone())
            );
            assert!(judge_v2(&query.checks, &query.user, &query.reply));
            for (_, value) in &stated {
                assert_eq!(
                    judge_v2(&query.checks, &query.user, &format!("It is {value}.")),
                    *value == answer,
                    "{value} for {answer}: {:?}",
                    conversation.turns
                );
            }
        }
        let episodes_drawn = all.len() as f64;
        let closed_share = closed as f64 / episodes_drawn;
        assert!(
            (closed_share - CLOSED_SHARE).abs() < 0.04,
            "{closed_share} of the episodes state a closed relation"
        );
        let abstain_share = abstentions as f64 / episodes_drawn;
        assert!(
            (abstain_share - 0.25).abs() < 0.04,
            "{abstain_share} of the episodes abstain"
        );
        // The companion is asked on a fair half of the recall queries ...
        let asked_share = companion_asked as f64 / recall as f64;
        assert!(recall > 1_500, "{recall}");
        assert!(
            (asked_share - 0.5).abs() < 0.05,
            "the companion is asked in {asked_share} of {recall} recall queries"
        );
        // ... whichever order the statements came in: the companion is stated
        // last in the designed share of the episodes that update (the update
        // is last in the rest), and the order does not tell the query.
        let updated_total = order[0] + order[1];
        let (numerator, denominator) = DISTRACTOR;
        let companion_last = order[0] as f64 / updated_total as f64;
        assert!(
            (companion_last - numerator as f64 / denominator as f64).abs() < 0.05,
            "the companion is stated last in {companion_last} of {updated_total} updated episodes"
        );
        assert!(order[1] > 200, "{order:?}");
        for (index, (items, asks)) in asked_by_order.iter().enumerate() {
            let share = *asks as f64 / *items as f64;
            assert!(
                (share - 0.5).abs() < 0.15,
                "order {index}: the companion is asked in {share} of {items} queries"
            );
        }
    }

    /// The replies of the query-blind rules over the values an episode states
    /// before its query: the latest value, the one before it, the first, the
    /// current value of the relation stated first and that of the other one.
    fn blind_replies(stated: &[(String, String)]) -> Vec<(&'static str, String)> {
        let value = |index: Option<usize>| -> String {
            index
                .and_then(|i| stated.get(i))
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        let last = stated.len().checked_sub(1);
        let queried = stated.first().map(|(name, _)| name.as_str());
        let current = |first_stated: bool| -> String {
            stated
                .iter()
                .rev()
                .find(|(name, _)| (Some(name.as_str()) == queried) == first_stated)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        vec![
            ("latest", value(last)),
            ("second-latest", value(stated.len().checked_sub(2).or(last))),
            ("first", value(Some(0))),
            ("queried-relation", current(true)),
            ("companion", current(false)),
        ]
    }

    #[test]
    fn no_query_blind_rule_over_the_stated_values_reaches_the_freeze_limit_on_relation_items() {
        // R-recency, the untrained rule the freeze condition reads (the latest
        // open value), on a fixed-seed sample of relation-only episodes.
        let mut world = only(0.0, 0.0, 1.0, 0.0);
        let mut rng = Rng::new(3_301);
        let runs = run_rules(&mut world, &mut rng, Cell::GATED, 2_400).expect("rule runs");
        let (pass, of) = runs[Rule::Recency.name()].card.cell("relation/open/recall");
        assert!(of > 1_000, "{of}");
        assert!(
            below_limit(pass, of),
            "R-recency passes {pass} of {of} open relation recall items"
        );
        // Balanced: the latest value is right about as often as it is wrong.
        assert!(pass * 5 > of * 2, "R-recency passes only {pass} of {of}");
        // Every rule over the stated values, on every recall item of both pools.
        let mut world = only(0.0, 0.0, 1.0, 0.0);
        let mut rng = Rng::new(3_302);
        let mut tallies: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
        for _ in 0..2_400 {
            let conversation = world
                .conversation_in(&mut rng, Cell::GATED)
                .expect("an episode");
            let Some((query, history)) = conversation.turns.split_last() else {
                continue;
            };
            if query.tag.abstain {
                continue;
            }
            for (name, reply) in blind_replies(&statements(history)) {
                let tally = tallies.entry(name).or_default();
                tally.0 += usize::from(judge_v2(&query.checks, &query.user, &reply));
                tally.1 += 1;
            }
        }
        assert_eq!(tallies.len(), 5);
        for (name, &(pass, of)) in &tallies {
            assert!(of > 1_500, "{name}: {of}");
            assert!(below_limit(pass, of), "{name} passes {pass} of {of}");
        }
        // The rules that name a relation, or the last statement, are each right
        // on about half the items: the query is the only thing that tells.
        for name in ["latest", "queried-relation", "companion"] {
            let (pass, of) = tallies[name];
            assert!(
                pass * 100 > of * 42 && pass * 100 < of * 58,
                "{name} passes {pass} of {of}"
            );
        }
    }

    #[test]
    fn cells_use_their_own_phrasings_and_values() {
        let train: BTreeSet<&str> = MWorld2::templates(Split::Train).into_iter().collect();
        let development: BTreeSet<&str> =
            MWorld2::templates(Split::Development).into_iter().collect();
        assert!(train.is_disjoint(&development));
        let closed_values = |split: Split| -> BTreeSet<&'static str> {
            relations()
                .iter()
                .filter_map(|r| match r.values {
                    Values::Closed(t, d) => Some(if split == Split::Train { t } else { d }),
                    _ => None,
                })
                .flatten()
                .copied()
                .collect()
        };
        // A town's split is its stem's; every other open value's is its own.
        let belongs = |value: &str, split: Split| -> bool {
            let lower = value.to_lowercase();
            split_of(&lower) == split
                || TOWN_SUFFIXES.iter().any(|suffix| {
                    lower
                        .strip_suffix(suffix)
                        .is_some_and(|stem| !stem.is_empty() && split_of(stem) == split)
                })
        };
        let mut open_values: BTreeMap<Split, BTreeSet<String>> = BTreeMap::new();
        let mut streams: BTreeMap<Cell, Vec<Conversation2>> = BTreeMap::new();
        for cell in Cell::ALL {
            let mut world = world();
            let mut rng = Rng::new(77);
            let mut checked = 0usize;
            for index in 0..600 {
                let conversation = world.conversation_in(&mut rng, cell).expect("an episode");
                if index < 30 {
                    streams.entry(cell).or_default().push(conversation.clone());
                }
                let (mine, other) = match cell.phrasing {
                    Split::Train => (&train, &development),
                    Split::Development => (&development, &train),
                };
                for turn in &conversation.turns {
                    if let Some(template) = &turn.tag.template {
                        checked += 1;
                        assert!(mine.contains(template.as_str()), "{cell:?} {template}");
                        assert!(!other.contains(template.as_str()), "{cell:?} {template}");
                    }
                    // Open values belong to the value split.
                    for value in &turn.tag.values {
                        assert!(belongs(value, cell.value), "{cell:?} {value}");
                        open_values
                            .entry(cell.value)
                            .or_default()
                            .insert(value.to_lowercase());
                    }
                    if let (Some(answer), Some(pool)) = (&turn.tag.answer, turn.tag.pool) {
                        match pool {
                            Pool::Open => {
                                assert!(belongs(answer, cell.value), "{cell:?} {answer}");
                            }
                            Pool::Closed => assert!(
                                closed_values(cell.value).contains(answer.as_str()),
                                "{cell:?} {answer}"
                            ),
                        }
                    }
                }
                // An MQAR lead-in is a template of the phrasing split, and
                // not of the other.
                if conversation.kind == Kind::Mqar {
                    let first = &conversation.turns[0].user;
                    let from = |p: &Phrasings, split: Split| {
                        let list = match split {
                            Split::Train => p.train,
                            Split::Development => p.development,
                        };
                        list.iter()
                            .any(|t| first.starts_with(t.split("{p}").next().unwrap_or("?")))
                    };
                    let other_split = match cell.phrasing {
                        Split::Train => Split::Development,
                        Split::Development => Split::Train,
                    };
                    assert!(from(&MQAR_LEAD, cell.phrasing), "{first}");
                    assert!(!from(&MQAR_LEAD, other_split), "{first}");
                }
            }
            assert!(checked > 300, "{cell:?} {checked}");
        }
        // No open value of one split appears in the other, across cells.
        let (train_values, development_values) = (
            &open_values[&Split::Train],
            &open_values[&Split::Development],
        );
        assert!(train_values.len() > 500 && development_values.len() > 500);
        assert!(train_values.is_disjoint(development_values));
        // The four cells are four different streams.
        let all: Vec<(&Cell, &Vec<Conversation2>)> = streams.iter().collect();
        assert_eq!(all.len(), 4);
        for (i, (a, first)) in all.iter().enumerate() {
            for (b, second) in &all[i + 1..] {
                assert_ne!(first, second, "{a:?} {b:?}");
            }
        }
    }

    /// `Cell::same(split)` is the plain split, checked on two renderings built
    /// apart: the plain path (a world from the default mix and the shared toy
    /// meter, rendered by [`render`]) and the cell path (a world from an
    /// explicit mix and its own token counter, drawn through
    /// [`MWorld2::conversation_in`] and rendered by a formatter written here).
    /// The texts, the document token counts and the SHA-256 of the whole
    /// rendered stream must all agree.
    #[test]
    fn the_same_split_cell_is_the_plain_split() {
        let by_cell_count = |text: &str| text.chars().count().div_ceil(3);
        let explicit = Mix {
            mqar: 0.35,
            copy: 0.10,
            relation: 0.30,
            other: 0.25,
            closed: CLOSED_SHARE,
        };
        let mut digests: BTreeMap<Split, String> = BTreeMap::new();
        for split in [Split::Train, Split::Development] {
            let plain = episodes(&mut world(), split, 91, 300);
            let plain_text: Vec<String> = plain.iter().map(|c| render(&c.turns)).collect();
            let plain_tokens: Vec<usize> = plain.iter().map(|c| c.tokens).collect();
            let mut cells = MWorld2::new(&by_cell_count, explicit).expect("a valid mix");
            let mut rng = Rng::new(91);
            let (mut cell_text, mut cell_tokens) = (Vec::new(), Vec::new());
            for _ in 0..300 {
                let conversation = cells
                    .conversation_in(&mut rng, Cell::same(split))
                    .expect("an episode");
                let mut text = String::new();
                for (index, turn) in conversation.turns.iter().enumerate() {
                    if index != 0 {
                        text.push('\n');
                    }
                    text.push_str("User: ");
                    text.push_str(&turn.user);
                    text.push_str("\nAssistant: ");
                    text.push_str(&turn.reply);
                }
                cell_text.push(text);
                cell_tokens.push(conversation.tokens);
            }
            assert_eq!(plain_text, cell_text, "{split:?}");
            assert_eq!(plain_tokens, cell_tokens, "{split:?}");
            let digest_of = |texts: &[String]| {
                let mut hasher = Sha256::new();
                for text in texts {
                    hasher.update(text.as_bytes());
                    hasher.update([0u8]);
                }
                hex::encode(hasher.finalize())
            };
            let digest = digest_of(&plain_text);
            assert_eq!(digest, digest_of(&cell_text), "{split:?}");
            digests.insert(split, digest);
        }
        // The two splits render different streams, and the development split
        // is the gated cell.
        assert_ne!(digests[&Split::Train], digests[&Split::Development]);
        assert_eq!(Cell::same(Split::Development), Cell::GATED);
        assert_eq!(Cell::ALL.len(), 4);
        let keys: BTreeSet<&str> = Cell::ALL.iter().map(|c| c.key()).collect();
        assert_eq!(keys.len(), 4);
    }

    #[test]
    fn the_a1_gate_is_unchanged_on_a_fixed_report() {
        // 27/30 at each distance and 18/20 on the open relation: exactly 0.9.
        let rows = |shortfall: Option<&str>| {
            let of = |key: &str, pass: usize, total: usize| {
                (
                    key.to_owned(),
                    if shortfall == Some(key) {
                        pass - 1
                    } else {
                        pass
                    },
                    total,
                )
            };
            [
                of("mqar/distance/16", 27, 30),
                of("mqar/distance/64", 27, 30),
                of("mqar/distance/200", 27, 30),
                of("relation/open/recall", 18, 20),
            ]
        };
        let build = |shortfall: Option<&str>| {
            let mut card = Scorecard::default();
            for (key, pass, total) in rows(shortfall) {
                for i in 0..total {
                    card.add(key.clone(), i < pass);
                }
            }
            card
        };
        let card = build(None);
        assert!(card.a1_gate());
        let report = card.to_json();
        assert_eq!(report["a1_gate"], json!(true));
        assert_eq!(
            report["a1_gate_rule"],
            json!("MQAR recall >= 0.9 at every distance (16, 64, 200) AND open-relation recall >= 0.9; an empty cell fails")
        );
        assert_eq!(
            report["mqar"]["by_distance"]["200"],
            json!({"pass": 27, "of": 30, "rate": 0.9})
        );
        assert_eq!(
            report["relation"]["open"],
            json!({"pass": 18, "of": 20, "rate": 0.9})
        );
        // One item fewer at any gated key fails the gate.
        for key in gated_keys() {
            assert!(!build(Some(key.as_str())).a1_gate(), "{key}");
        }
        // The cells decide on the development cell alone.
        let mut cells = CellScores::default();
        for cell in Cell::ALL {
            let pass = cell == Cell::PURE_RETRIEVAL;
            for d in DISTANCES {
                for _ in 0..10 {
                    cells.record(cell, &mqar_item(d), pass);
                }
            }
            for _ in 0..10 {
                cells.record(cell, &recall_item(Pool::Open), pass);
            }
        }
        assert!(!cells.a1_gate(), "only the pure-retrieval cell passes");
        let mut gated = CellScores::default();
        for d in DISTANCES {
            for _ in 0..10 {
                gated.record(Cell::GATED, &mqar_item(d), true);
            }
        }
        for _ in 0..10 {
            gated.record(Cell::GATED, &recall_item(Pool::Open), true);
        }
        assert!(gated.a1_gate());
        assert!(!CellScores::default().a1_gate());
        let report = cells.to_json();
        assert_eq!(report["a1_gate"], json!(false));
        assert_eq!(report["gated_cell"], json!("dev_phrasing_dev_value"));
        assert_eq!(
            report["pure_retrieval"]["cell"],
            json!("train_phrasing_dev_value")
        );
        assert_eq!(
            report["pure_retrieval"]["scores"]["mqar/distance/64"]["rate"],
            json!(1.0)
        );
        assert_eq!(
            report["matrix"]["dev_phrasing_dev_value"]["relation/open/recall"]["pass"],
            json!(0)
        );
        // The gate and its rule appear on the development cell alone.
        assert!(report["cells"]["dev_phrasing_dev_value"]
            .get("a1_gate")
            .is_some());
        assert!(report["cells"]["train_phrasing_dev_value"]
            .get("a1_gate")
            .is_none());
        assert!(report["cells"]["train_phrasing_train_value"]
            .get("a1_gate_rule")
            .is_none());
    }

    #[test]
    fn the_syllable_universe_test_matches_the_generator() {
        let mut rng = Rng::new(4);
        for split in [Split::Train, Split::Development] {
            for _ in 0..400 {
                let w = word(&mut rng, split, 1, 3).expect("a word");
                assert!(is_syllable_word(&w, 3) && in_generated_universe(&w), "{w}");
                let n = name(&mut rng, split).expect("a name");
                assert!(in_generated_universe(&n), "{n}");
                let t = town(&mut rng, split).expect("a town");
                assert!(in_generated_universe(&t), "{t}");
                let d = digits(&mut rng, split).expect("digits");
                assert!(in_generated_universe(&d), "{d}");
            }
        }
        for outside in [
            "Clara", "Ingrid", "Yusuf", "Halifax", "Okafor", "UA772", "48213", "12B", "5", "918264",
        ] {
            assert!(!in_generated_universe(outside), "{outside}");
        }
        for inside in ["Denver", "Boulder", "kavu", "4821", "Bolville"] {
            assert!(in_generated_universe(inside), "{inside}");
        }
        assert!(!is_syllable_word("", 3) && !is_syllable_word("kavu", 1));
        assert!(is_syllable_word("kavu", 2));
    }

    // -- The sealed probe and the position rules (#1541, item 2) -------------

    /// The replies of the query-blind position rules over the values a probe
    /// item's context states, in reading order: the first three and the last
    /// three. With fewer values, the k-th from the start takes the last and
    /// the k-th from the end the first.
    fn probe_position_replies(item: &ProbeItem) -> Vec<(&'static str, String)> {
        let context = &item.turns[..item.turns.len().saturating_sub(1)];
        let stated: Vec<&str> = context
            .iter()
            .flat_map(|turn| turn.values.iter().map(String::as_str))
            .collect();
        let from_start = |k: usize| -> String {
            stated
                .get(k - 1)
                .or(stated.last())
                .copied()
                .unwrap_or_default()
                .to_owned()
        };
        let from_end = |k: usize| -> String {
            stated
                .len()
                .checked_sub(k)
                .and_then(|i| stated.get(i))
                .or(stated.first())
                .copied()
                .unwrap_or_default()
                .to_owned()
        };
        vec![
            ("first", from_start(1)),
            ("second", from_start(2)),
            ("third", from_start(3)),
            ("latest", from_end(1)),
            ("second-latest", from_end(2)),
            ("third-latest", from_end(3)),
        ]
    }

    /// (pass, of) of each position rule over the recall items of `items`
    /// (every item that is not a copy), judged by the item's own checks.
    fn probe_position_rates(items: &[ProbeItem]) -> BTreeMap<&'static str, (usize, usize)> {
        let mut tallies: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
        for item in items.iter().filter(|item| item.group != ProbeGroup::Copy) {
            let checks = item.checks().expect("the item's checks");
            let scored = item.turns.last().expect("a scored turn");
            for (name, reply) in probe_position_replies(item) {
                let tally = tallies.entry(name).or_default();
                tally.0 += usize::from(judge_v2(&checks, &scored.user, &reply));
                tally.1 += 1;
            }
        }
        tallies
    }

    #[test]
    fn the_position_rules_are_read_off_the_stated_values() {
        let turn =
            |user: &str, reply: &str, values: &[&str], answer: Option<&str>, stale: &[&str]| {
                ProbeTurn {
                    user: user.to_owned(),
                    reply: reply.to_owned(),
                    values: values.iter().map(|v| (*v).to_owned()).collect(),
                    answer: answer.map(str::to_owned),
                    stale: stale.iter().map(|v| (*v).to_owned()).collect(),
                    copy: None,
                }
            };
        // Four values stated and the third asked: only the rules that land on
        // the third value pass.
        let four = ProbeItem {
            id: "mqar-short-99".into(),
            group: ProbeGroup::MqarShort,
            turns: vec![
                turn(
                    "Notes: alpha is kavu, beta is nomu, gamma is tesa, delta is rilo.",
                    "Noted.",
                    &["kavu", "nomu", "tesa", "rilo"],
                    None,
                    &[],
                ),
                turn(
                    "Which one goes with gamma?",
                    "Gamma goes with tesa.",
                    &[],
                    Some("tesa"),
                    &["kavu", "nomu", "rilo"],
                ),
            ],
        };
        // One value stated: every rule falls back to it.
        let one = ProbeItem {
            id: "relation-99".into(),
            group: ProbeGroup::Relation,
            turns: vec![
                turn("My cafe is Halcyon.", "Nice.", &["Halcyon"], None, &[]),
                turn(
                    "Which cafe is mine?",
                    "Your cafe is Halcyon.",
                    &[],
                    Some("Halcyon"),
                    &[],
                ),
            ],
        };
        // A copy item has no recall rule to run.
        let copy = ProbeItem {
            id: "copy-99".into(),
            group: ProbeGroup::Copy,
            turns: vec![ProbeTurn {
                user: "Repeat: alpha beta".into(),
                reply: "alpha beta".into(),
                values: Vec::new(),
                answer: None,
                stale: Vec::new(),
                copy: Some("alpha beta".into()),
            }],
        };
        let rates = probe_position_rates(&[four, one, copy]);
        assert_eq!(rates.len(), 6);
        for (name, &(pass, of)) in &rates {
            assert_eq!(of, 2, "{name}");
            let expected = match *name {
                "third" | "second-latest" => 2,
                _ => 1,
            };
            assert_eq!(pass, expected, "{name}");
        }
    }

    /// No query-blind rule over the values the context states (the first
    /// three, the last three) may reach [`FREEZE_LIMIT`] on the probe's recall
    /// items: the item is answered by reading the query. The probe file as
    /// first sealed (a query-blind second-latest rule scored 22 of 30) fails
    /// this, and the rebalanced file must pass it.
    ///
    /// The rebalanced file (the first sealed one with [`REBALANCED_ROWS`]
    /// applied) is installed as `data/a1-english-probe.json` and pinned by
    /// `PROBE_SHA256` in `milestone_world_v2_probe` (#1541, item 2), so this
    /// test holds the installed file to the bound.
    #[test]
    fn no_query_blind_position_rule_reaches_the_freeze_limit_on_the_probe() {
        let items = probe().expect("the probe loads and validates");
        let rates = probe_position_rates(&items);
        assert_eq!(rates.len(), 6);
        let mut leaks = Vec::new();
        for (name, &(pass, of)) in &rates {
            assert_eq!(of, 30, "{name}: the probe has 30 recall items");
            if !below_limit(pass, of) {
                leaks.push(format!("{name} {pass}/{of}"));
            }
        }
        assert!(
            leaks.is_empty(),
            "the installed probe is position-biased ({leaks:?}): a query-blind position \
             rule reaches the freeze limit on data/a1-english-probe.json"
        );
    }

    /// The seven scored turns that rebalance the sealed probe (#1541, item 2),
    /// by item id: (id, user, reply, answer, stale). Each asks a different fact
    /// of the context the item already has, so the answer moves along the
    /// stated values (one asks the fourth of four, one the first of four, four
    /// the third of three, one the second of two) and the context turns stay as
    /// they were sealed. They are installed in `data/a1-english-probe.json`
    /// (pinned by `PROBE_SHA256`), so this table repeats the file; the test
    /// below applies it in memory, which leaves the installed items unchanged.
    const REBALANCED_ROWS: [(&str, &str, &str, &str, &[&str]); 7] = [
        (
            "mqar-short-06",
            "Who is bringing the folding tables?",
            "Ottoline is bringing the folding tables.",
            "Ottoline",
            &["Wendell"],
        ),
        (
            "mqar-short-07",
            "What was the storage code again?",
            "The storage code is 77120.",
            "77120",
            &["Savannah", "Zainab"],
        ),
        (
            "mqar-long-03",
            "Who runs registration?",
            "Cormac runs registration.",
            "Cormac",
            &["Delphine", "Ottmar", "Isaac"],
        ),
        (
            "mqar-long-04",
            "Who was her manager again?",
            "Her manager is Petrov.",
            "Petrov",
            &["Fairbanks", "47718"],
        ),
        (
            "mqar-long-06",
            "Where is the April book set?",
            "The April book is set in Yakima.",
            "Yakima",
            &["Kyoto", "Nagoya", "Sapporo"],
        ),
        (
            "relation-04",
            "What was the building code again?",
            "The building code is 66041.",
            "66041",
            &["20983", "54167"],
        ),
        (
            "relation-11",
            "Who runs the florist these days?",
            "Gwen runs the florist.",
            "Gwen",
            &["Imogen", "Quentin"],
        ),
    ];

    /// The probe with [`REBALANCED_ROWS`] applied in memory meets every bound
    /// the sealed one misses: no position rule reaches the freeze limit, each
    /// reference reply still passes its checks and names only stated values,
    /// and neither untrained rule reaches one half in any group (the bound
    /// `milestone_world_v2_probe` holds the file to).
    #[test]
    fn the_rebalanced_probe_rows_meet_the_position_bound() {
        let mut items = probe().expect("the probe loads and validates");
        for (id, user, reply, answer, stale) in REBALANCED_ROWS {
            let item = items
                .iter_mut()
                .find(|item| item.id == id)
                .expect("a probe item of that id");
            let scored = item.turns.last_mut().expect("a scored turn");
            scored.user = user.to_owned();
            scored.reply = reply.to_owned();
            scored.answer = Some(answer.to_owned());
            scored.stale = stale.iter().map(|s| (*s).to_owned()).collect();
        }
        for item in items.iter().filter(|item| item.group != ProbeGroup::Copy) {
            let checks = item.checks().expect("the item's checks");
            let scored = item.turns.last().expect("a scored turn");
            assert!(
                judge_v2(&checks, &scored.user, &scored.reply),
                "{}",
                item.id
            );
            assert!(
                !judge_v2(&checks, &scored.user, &scored.user),
                "{}",
                item.id
            );
            let stated: BTreeSet<&str> = item.turns[..item.turns.len() - 1]
                .iter()
                .flat_map(|turn| turn.values.iter().map(String::as_str))
                .collect();
            let mut named = scored.answer.iter().chain(&scored.stale);
            assert!(
                named.all(|value| stated.contains(value.as_str())),
                "{}",
                item.id
            );
        }
        let rates = probe_position_rates(&items);
        assert_eq!(rates.len(), 6);
        for (name, &(pass, of)) in &rates {
            assert_eq!(of, 30, "{name}");
            assert!(below_limit(pass, of), "{name} passes {pass} of {of}");
        }
        let report = static_report(&items, &Meter::new(&toy), CONTEXT).expect("a report");
        for (group, cell) in report["by_group"].as_object().expect("groups") {
            for (rule, row) in cell["rules"].as_object().expect("rules") {
                let rate = row["rate"].as_f64().expect("a rate");
                assert!(rate < 0.5, "{rule} on {group}: {rate}");
            }
        }
    }

    /// The real tokenizer: every episode of every kind fits 256 tokens, the
    /// meter equals the protocol's own encoding, and the MQAR distances are
    /// what the encoder produces. Run with `--ignored` (set UOR_R4_TOKENIZER to
    /// use another tokenizer).
    #[test]
    #[ignore = "needs the issue-1017 tokenizer JSON"]
    fn real_tokenizer_episodes_fit_the_context() {
        use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
        use uor_r4_tokenizer::ByteBpeTokenizer;
        let path = std::env::var("UOR_R4_TOKENIZER").unwrap_or_else(|_| {
            "/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/tokenizer/tokenizer.json"
                .into()
        });
        let bytes = std::fs::read(path).expect("the tokenizer file");
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).expect("a tokenizer");
        let protocol = DialogueProtocol::literal_roles_v1(&tokenizer).expect("a protocol");
        let encoder = protocol.bind(&tokenizer).expect("an encoder");
        let count = |text: &str| tokenizer.encode(text).len();
        // Version 2 encodes the same conversations; its own meter counts them.
        let spaced_protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("version 2");
        let spaced_encoder = spaced_protocol.bind(&tokenizer).expect("an encoder");
        let spaced = Meter::spaced(&count);
        let mut world = MWorld2::new(&count, Mix::default()).expect("a world");
        let mut longest = 0;
        for split in [Split::Train, Split::Development] {
            let mut rng = Rng::new(61);
            for _ in 0..1500 {
                let conversation = world.conversation(&mut rng, split).expect("an episode");
                let messages: Vec<Message<'_>> = conversation
                    .turns
                    .iter()
                    .flat_map(|t| {
                        [
                            Message {
                                role: "user",
                                content: &t.user,
                            },
                            Message {
                                role: "assistant",
                                content: &t.reply,
                            },
                        ]
                    })
                    .collect();
                let encoded = encoder.encode_document(&messages);
                assert_eq!(
                    encoded.tokens.len(),
                    conversation.tokens,
                    "{:?}",
                    conversation.turns
                );
                assert_eq!(
                    spaced_encoder.encode_document(&messages).tokens.len(),
                    spaced.document(&conversation.turns),
                    "{:?}",
                    conversation.turns
                );
                assert!(encoded.tokens.len() <= CONTEXT);
                longest = longest.max(encoded.tokens.len());
                if let Some(mqar) = conversation.turns.last().and_then(|t| t.tag.mqar.as_ref()) {
                    // From the end of the assertion turn to the start of the
                    // query text, as the encoder lays the tokens out.
                    let last = conversation.turns.len() - 1;
                    let head = encoder.encode_open_history(&messages[..1]).tokens.len();
                    let to_query = encoder
                        .encode_open_history(&messages[..2 * last + 1])
                        .tokens
                        .len()
                        - count(conversation.turns[last].user.trim());
                    assert_eq!(to_query - head, mqar.distance);
                    assert!(mqar.distance >= mqar.target_distance);
                    assert!(
                        mqar.distance <= mqar.target_distance + tolerance(mqar.target_distance)
                    );
                }
            }
        }
        println!("longest episode: {longest} tokens");
    }

    #[test]
    fn training_paraphrases_are_validated_and_only_change_training_picks() {
        let pair = |s: &str, t: &str| (s.to_owned(), t.to_owned());
        let table = train_paraphrases(
            &[
                pair(
                    "What letter does {x} start with?",
                    "Which letter starts {x}?",
                ),
                pair("What is {a} plus {b}?", "How much is {a} and {b} together?"),
            ],
            1000,
        )
        .unwrap();
        assert_eq!(table.by_template.len(), 2);
        // Rejected: a changed slot, an unknown source, a world template as
        // text, a development source and a share above one.
        for (source, text) in [
            ("What is {a} plus {b}?", "How much is {a} and seven?"),
            ("Tell me a joke.", "Say something funny."),
            ("What is {a} plus {b}?", "Sum {a} and {b}."),
            ("Sum {a} and {b}.", "Total {a} with {b}."),
        ] {
            assert!(
                train_paraphrases(&[pair(source, text)], 500).is_err(),
                "{source}"
            );
        }
        assert!(train_paraphrases(&[], 1001).is_err());

        let first_letter = Phrasings {
            train: &["What letter does {x} start with?"],
            development: &["{x} begins with which letter?"],
        };
        let mut rng = Rng::new(7);
        assert_eq!(
            first_letter.pick_with(&mut rng, Split::Train, Some(&table)),
            "Which letter starts {x}?"
        );
        // Development picks, and training templates without wordings, draw
        // exactly as without paraphrases.
        let other = Phrasings {
            train: &["Spell the word {x}.", "How do you spell {x}?"],
            development: &["What are the letters in {x}?"],
        };
        for (phrasings, split) in [
            (&first_letter, Split::Development),
            (&other, Split::Train),
            (&other, Split::Development),
        ] {
            let (mut a, mut b) = (Rng::new(11), Rng::new(11));
            for _ in 0..20 {
                assert_eq!(
                    phrasings.pick_with(&mut a, split, Some(&table)),
                    phrasings.pick_with(&mut b, split, None)
                );
            }
        }
        // A zero share keeps every template.
        let none = train_paraphrases(
            &[pair(
                "What letter does {x} start with?",
                "Which letter starts {x}?",
            )],
            0,
        )
        .unwrap();
        assert_eq!(
            first_letter.pick_with(&mut rng, Split::Train, Some(&none)),
            "What letter does {x} start with?"
        );
    }
}
